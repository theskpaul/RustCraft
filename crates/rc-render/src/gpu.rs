use anyhow::{Context, Result, bail};

/// Higher is better. CPU (software) adapters rank lowest.
pub fn rank(t: wgpu::DeviceType) -> u8 {
    match t {
        wgpu::DeviceType::DiscreteGpu => 4,
        wgpu::DeviceType::IntegratedGpu => 3,
        wgpu::DeviceType::VirtualGpu => 2,
        wgpu::DeviceType::Other => 1,
        wgpu::DeviceType::Cpu => 0,
    }
}

fn backends_from_env() -> Result<wgpu::Backends> {
    Ok(match std::env::var("RC_BACKEND").as_deref() {
        Err(_) | Ok("vulkan") => wgpu::Backends::VULKAN,
        Ok("gl") => wgpu::Backends::GL,
        Ok("all") => wgpu::Backends::all(),
        Ok(other) => bail!("RC_BACKEND must be vulkan|gl|all, got '{other}'"),
    })
}

fn choose_present_mode(caps: &wgpu::SurfaceCapabilities) -> wgpu::PresentMode {
    let want = match std::env::var("RC_PRESENT").as_deref() {
        Ok("immediate") => wgpu::PresentMode::Immediate,
        Ok("mailbox") => wgpu::PresentMode::Mailbox,
        _ => wgpu::PresentMode::Fifo,
    };
    if caps.present_modes.contains(&want) {
        want
    } else {
        log::warn!(
            "present mode {want:?} unsupported (supported: {:?}); using Fifo",
            caps.present_modes
        );
        wgpu::PresentMode::Fifo
    }
}

pub struct GpuContext {
    pub surface: wgpu::Surface<'static>,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,
    pub is_software: bool,
}

impl GpuContext {
    /// `target` is typically an `Arc<winit::window::Window>`.
    pub fn new(target: impl Into<wgpu::SurfaceTarget<'static>>, size: (u32, u32)) -> Result<Self> {
        let backends = backends_from_env()?;
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends,
            ..Default::default()
        });
        let surface = instance
            .create_surface(target)
            .context("create_surface failed")?;

        let adapters = instance.enumerate_adapters(backends);
        if adapters.is_empty() {
            bail!("no GPU adapters found for backends {backends:?}");
        }
        for (i, a) in adapters.iter().enumerate() {
            let info = a.get_info();
            log::info!(
                "adapter[{i}]: '{}' type={:?} backend={:?} pci={:04x}:{:04x} driver='{}' ({}) surface_ok={}",
                info.name,
                info.device_type,
                info.backend,
                info.vendor,
                info.device,
                info.driver,
                info.driver_info,
                a.is_surface_supported(&surface),
            );
        }

        let want = std::env::var("RC_ADAPTER").ok().map(|s| s.to_lowercase());
        let mut cands: Vec<wgpu::Adapter> = adapters
            .into_iter()
            .filter(|a| a.is_surface_supported(&surface))
            .filter(|a| {
                want.as_ref()
                    .is_none_or(|w| a.get_info().name.to_lowercase().contains(w))
            })
            .collect();
        cands.sort_by_key(|a| std::cmp::Reverse(rank(a.get_info().device_type)));
        let adapter = cands
            .into_iter()
            .next()
            .context("no adapter supports the surface (or RC_ADAPTER matched nothing)")?;

        let info = adapter.get_info();
        let is_software = info.device_type == wgpu::DeviceType::Cpu
            || info.name.to_lowercase().contains("llvmpipe");
        if is_software {
            if std::env::var("RC_ALLOW_SOFTWARE").as_deref() == Ok("1") {
                log::warn!("!!! SOFTWARE RENDERER SELECTED ('{}') !!!", info.name);
            } else {
                bail!(
                    "selected adapter '{}' is a software renderer; refusing. \
                     Set RC_ALLOW_SOFTWARE=1 to allow.",
                    info.name
                );
            }
        }
        log::info!(
            "SELECTED: '{}' type={:?} backend={:?}",
            info.name,
            info.device_type,
            info.backend
        );
        let l = adapter.limits();
        log::info!(
            "limits: max_texture_2d={} max_buffer_size={} max_storage_binding={} max_bind_groups={}",
            l.max_texture_dimension_2d,
            l.max_buffer_size,
            l.max_storage_buffer_binding_size,
            l.max_bind_groups
        );
        log::debug!("features: {:?}", adapter.features());

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("rustcraft-device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            ..Default::default()
        }))
        .context("request_device failed")?;
        device.set_device_lost_callback(|reason, msg| {
            log::error!("DEVICE LOST: {reason:?}: {msg}");
        });

        let caps = surface.get_capabilities(&adapter);
        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);
        let present_mode = choose_present_mode(&caps);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width: size.0.max(1),
            height: size.1.max(1),
            present_mode,
            desired_maximum_frame_latency: 2,
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        log::info!(
            "surface: {}x{} format={:?} present={:?}",
            config.width,
            config.height,
            config.format,
            config.present_mode
        );

        Ok(Self {
            surface,
            adapter,
            device,
            queue,
            config,
            is_software,
        })
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 {
            return; // minimized
        }
        self.config.width = w;
        self.config.height = h;
        self.surface.configure(&self.device, &self.config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wgpu::DeviceType::*;

    #[test]
    fn software_ranks_lowest_and_hardware_first() {
        assert!(rank(DiscreteGpu) > rank(IntegratedGpu));
        assert!(rank(IntegratedGpu) > rank(VirtualGpu));
        assert!(rank(VirtualGpu) > rank(Other));
        assert!(rank(Other) > rank(Cpu));
    }
}
