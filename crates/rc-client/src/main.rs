mod fly;
mod stats;

use anyhow::{Context, Result};
use rc_core::{BlockId, Chunk};
use rc_render::camera::Camera;
use rc_render::chunk_renderer::ChunkRenderer;
use rc_render::mesher::mesh_chunk;
use rc_render::{FrameOutcome, GpuContext, render_chunk_frame};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorGrabMode, Window, WindowId};

struct State {
    window: Arc<Window>,
    gpu: GpuContext,
    chunks: ChunkRenderer,
    camera: Camera,
    fly: fly::FlyInput,
    grabbed: bool,
    stats: stats::FrameStats,
    started: Instant,
    last_frame: Instant,
    exit_after: Option<Duration>,
}

/// Deterministic blocky terrain for the 2a visual test.
fn test_chunk() -> Chunk {
    let mut c = Chunk::new_empty();
    for z in 0..16usize {
        for x in 0..16usize {
            let h = 5 + (x / 4 + z / 4) % 3;
            for y in 0..h {
                let b = if y + 1 == h {
                    BlockId::GRASS
                } else if y + 4 >= h {
                    BlockId::DIRT
                } else {
                    BlockId::STONE
                };
                c.set(x, y, z, b);
            }
        }
    }
    for y in 8..13 {
        c.set(8, y, 8, BlockId::STONE); // pillar, checks depth ordering
    }
    c
}

impl State {
    fn new(el: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("RustCraft - M2a")
                    .with_inner_size(LogicalSize::new(1280.0, 720.0)),
            )
            .context("create_window failed")?,
        );
        let size = window.inner_size();
        let gpu = GpuContext::new(window.clone(), (size.width, size.height))?;

        let mut chunks =
            ChunkRenderer::new(&gpu.device, gpu.config.format, (size.width, size.height));
        let mesh = mesh_chunk(&test_chunk());
        log::info!(
            "test chunk: {} faces, {} vertices, {} indices",
            mesh.face_count(),
            mesh.vertices.len(),
            mesh.indices.len()
        );
        chunks.upload_chunk(&gpu.device, &mesh, [0, 0, 0]);

        let aspect = size.width.max(1) as f32 / size.height.max(1) as f32;
        let exit_after = std::env::var("RC_EXIT_AFTER")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Duration::from_secs_f64);
        let now = Instant::now();
        Ok(Self {
            window,
            gpu,
            chunks,
            camera: Camera::new(aspect),
            fly: fly::FlyInput::default(),
            grabbed: false,
            stats: stats::FrameStats::new(),
            started: now,
            last_frame: now,
            exit_after,
        })
    }

    fn set_grab(&mut self, on: bool) {
        if on {
            let ok = self.window.set_cursor_grab(CursorGrabMode::Locked).is_ok()
                || self
                    .window
                    .set_cursor_grab(CursorGrabMode::Confined)
                    .is_ok();
            if !ok {
                log::warn!("cursor grab not supported on this window system");
                return;
            }
            self.window.set_cursor_visible(false);
        } else {
            let _ = self.window.set_cursor_grab(CursorGrabMode::None);
            self.window.set_cursor_visible(true);
            self.fly = fly::FlyInput::default();
        }
        self.grabbed = on;
    }

    fn set_key(&mut self, code: KeyCode, down: bool) {
        match code {
            KeyCode::KeyW => self.fly.forward = down,
            KeyCode::KeyS => self.fly.back = down,
            KeyCode::KeyA => self.fly.left = down,
            KeyCode::KeyD => self.fly.right = down,
            KeyCode::Space => self.fly.up = down,
            KeyCode::ControlLeft => self.fly.down = down,
            KeyCode::ShiftLeft => self.fly.boost = down,
            _ => {}
        }
    }

    fn print_overall(&self) {
        if let Some(r) = self.stats.overall() {
            log::info!(
                "SUMMARY: frames={} avg={:.2}ms ({:.1} fps) p50={:.2}ms p99={:.2}ms (1% low {:.1} fps) max={:.2}ms software={}",
                r.frames,
                r.avg_ms,
                r.fps(),
                r.p50_ms,
                r.p99_ms,
                r.low1_fps(),
                r.max_ms,
                self.gpu.is_software
            );
        }
    }
}

#[derive(Default)]
struct App {
    state: Option<State>,
    error: Option<anyhow::Error>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, el: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }
        match State::new(el) {
            Ok(s) => self.state = Some(s),
            Err(e) => {
                log::error!("startup failed: {e:#}");
                self.error = Some(e);
                el.exit();
            }
        }
    }

    fn device_event(&mut self, _el: &ActiveEventLoop, _id: DeviceId, event: DeviceEvent) {
        let Some(s) = self.state.as_mut() else { return };
        if let DeviceEvent::MouseMotion { delta } = event
            && s.grabbed
        {
            s.fly.look.0 += delta.0 as f32;
            s.fly.look.1 += delta.1 as f32;
        }
    }

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(s) = self.state.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => {
                s.print_overall();
                el.exit();
            }
            WindowEvent::KeyboardInput { event, .. } => {
                let down = event.state == ElementState::Pressed;
                if let PhysicalKey::Code(code) = event.physical_key {
                    if down && !event.repeat && code == KeyCode::Escape {
                        // First Esc releases the mouse, second Esc exits.
                        if s.grabbed {
                            s.set_grab(false);
                        } else {
                            s.print_overall();
                            el.exit();
                        }
                    } else {
                        s.set_key(code, down);
                    }
                }
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } if !s.grabbed => {
                s.set_grab(true);
            }
            WindowEvent::Focused(false) => s.set_grab(false),
            WindowEvent::Resized(size) => {
                s.gpu.resize(size.width, size.height);
                s.chunks.resize(&s.gpu.device, (size.width, size.height));
                if size.width > 0 && size.height > 0 {
                    s.camera.aspect = size.width as f32 / size.height as f32;
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let dt = (now - s.last_frame).as_secs_f32().min(0.1);
                s.last_frame = now;
                fly::update(&mut s.camera, &mut s.fly, dt);

                match render_chunk_frame(&s.gpu, &s.chunks, s.camera.view_proj()) {
                    Ok(FrameOutcome::Presented) => {
                        if let Some(r) = s.stats.tick() {
                            log::info!(
                                "frame: {} frames, avg {:.2}ms ({:.1} fps), p50 {:.2}, p99 {:.2}, max {:.2}",
                                r.frames,
                                r.avg_ms,
                                r.fps(),
                                r.p50_ms,
                                r.p99_ms,
                                r.max_ms
                            );
                            s.window.set_title(&format!(
                                "RustCraft M2a | {:.1} fps | {:.2} ms",
                                r.fps(),
                                r.avg_ms
                            ));
                        }
                    }
                    Ok(other) => log::debug!("frame outcome: {other:?}"),
                    Err(e) => {
                        log::error!("render error: {e:#}");
                        self.error = Some(e);
                        el.exit();
                        return;
                    }
                }
                if let Some(limit) = s.exit_after
                    && s.started.elapsed() >= limit
                {
                    s.print_overall();
                    el.exit();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        if let Some(s) = &self.state {
            s.window.request_redraw();
        }
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info,wgpu_core=warn,wgpu_hal=warn,naga=warn"),
    )
    .init();

    let event_loop = EventLoop::new().context("EventLoop::new failed")?;
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::default();
    event_loop.run_app(&mut app).context("event loop error")?;
    match app.error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}
