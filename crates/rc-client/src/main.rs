mod stats;

use anyhow::{Context, Result};
use rc_render::{FrameOutcome, GpuContext, TrianglePipeline, render_frame};
use std::sync::Arc;
use std::time::{Duration, Instant};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowId};

struct State {
    window: Arc<Window>,
    gpu: GpuContext,
    tri: TrianglePipeline,
    stats: stats::FrameStats,
    started: Instant,
    exit_after: Option<Duration>,
}

impl State {
    fn new(el: &ActiveEventLoop) -> Result<Self> {
        let window = Arc::new(
            el.create_window(
                Window::default_attributes()
                    .with_title("RustCraft - M1")
                    .with_inner_size(LogicalSize::new(1280.0, 720.0)),
            )
            .context("create_window failed")?,
        );
        let size = window.inner_size();
        let gpu = GpuContext::new(window.clone(), (size.width, size.height))?;
        let tri = TrianglePipeline::new(&gpu.device, gpu.config.format)?;
        let exit_after = std::env::var("RC_EXIT_AFTER")
            .ok()
            .and_then(|s| s.parse::<f64>().ok())
            .map(Duration::from_secs_f64);
        Ok(Self {
            window,
            gpu,
            tri,
            stats: stats::FrameStats::new(),
            started: Instant::now(),
            exit_after,
        })
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

    fn window_event(&mut self, el: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(s) = self.state.as_mut() else { return };
        match event {
            WindowEvent::CloseRequested => {
                s.print_overall();
                el.exit();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed
                    && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                s.print_overall();
                el.exit();
            }
            WindowEvent::Resized(size) => s.gpu.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                match render_frame(&s.gpu, &s.tri) {
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
                                "RustCraft M1 | {:.1} fps | {:.2} ms",
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
