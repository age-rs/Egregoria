use crate::audio::AudioContext;
use crate::game_loop;
use crate::input::InputContext;
use egregoria::utils::time::GameTime;
use futures::executor;
use geom::{vec2, vec3};
use std::time::Instant;
use wgpu_engine::GfxContext;
use winit::window::Window;
use winit::{
    dpi::PhysicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub struct Context {
    pub gfx: GfxContext,
    pub input: InputContext,
    pub audio: AudioContext,
    pub window: Window,
    pub el: Option<EventLoop<()>>,
    pub delta: f32,
}

impl Context {
    pub fn new() -> Self {
        let el = EventLoop::new();

        let size = el
            .primary_monitor()
            .expect("app needs a monitor to run")
            .size();

        let wb = WindowBuilder::new();

        let window;
        #[cfg(target_os = "windows")]
        {
            use winit::platform::windows::WindowBuilderExtWindows;
            window = wb.with_drag_and_drop(false);
        }
        #[cfg(not(target_os = "windows"))]
        {
            window = wb;
        }
        let window = window
            .with_inner_size(PhysicalSize::new(
                size.width as f32 * 0.8,
                size.height as f32 * 0.8,
            ))
            .with_title(format!("Egregoria {}", goria_version::VERSION))
            .build(&el)
            .expect("Failed to create window");

        let gfx = executor::block_on(GfxContext::new(
            &window,
            window.inner_size().width,
            window.inner_size().height,
        ));
        let input = InputContext::default();
        let audio = AudioContext::new();

        Self {
            gfx,
            input,
            audio,
            window,
            el: Some(el),
            delta: 0.0,
        }
    }

    pub fn start(mut self, mut state: game_loop::State) {
        let mut frame: Option<_> = None;
        let mut new_size: Option<PhysicalSize<u32>> = None;
        let mut last_update = Instant::now();

        self.el.take().unwrap().run(move |event, _, control_flow| {
            *control_flow = ControlFlow::Poll;
            state.event(&self.window, &event);
            match event {
                Event::WindowEvent { event, .. } => {
                    let managed = self.input.handle(&event);

                    if !managed {
                        match event {
                            WindowEvent::Resized(physical_size) => {
                                log::info!("resized: {:?}", physical_size);
                                new_size = Some(physical_size);
                                frame.take();
                            }
                            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                            _ => (),
                        }
                    }
                }
                Event::MainEventsCleared => match frame.take() {
                    None => {
                        if let Some(new_size) = new_size.take() {
                            self.gfx.resize(new_size.width, new_size.height);
                            state.resized(&mut self, new_size);
                            self.gfx.update_sc = false;
                        }

                        let size = self.gfx.size;
                        if self.gfx.update_sc {
                            self.gfx.update_sc = false;
                            self.gfx.resize(size.0, size.1);
                        }

                        match self.gfx.fbos.swapchain.get_current_frame() {
                            Ok(swapchainframe) => {
                                frame = Some(swapchainframe);
                            }
                            Err(wgpu_engine::wgpu::SwapChainError::Outdated)
                            | Err(wgpu_engine::wgpu::SwapChainError::Lost) => {
                                self.gfx.resize(size.0, size.1);
                                state.resized(&mut self, PhysicalSize::new(size.0, size.1));
                            }
                            Err(e) => panic!("error getting swapchain: {}", e),
                        };
                    }
                    Some(sco) => {
                        self.input.mouse.unprojected =
                            state.camera.unproject(self.input.mouse.screen);

                        let d = last_update.elapsed();
                        last_update = Instant::now();
                        self.delta = d.as_secs_f32();
                        state.update(&mut self);

                        let window = &self.window;
                        let mut enc = self.gfx.start_frame();
                        self.gfx.render_objs(&mut enc, |fc| state.render(fc));

                        let (lights, ambiant_col) = state.lights();

                        let t = std::f32::consts::TAU
                            * (self.gfx.render_params.value().time - 8.0 * GameTime::HOUR as f32)
                            / GameTime::DAY as f32;

                        let sun = vec3(t.cos(), t.sin() * 0.5, t.sin() + 0.5).normalize();

                        let params = self.gfx.render_params.value_mut();
                        params.ambiant = ambiant_col;
                        params.cam_pos = state.camera.camera.eye();
                        params.sun = sun;
                        params.viewport = vec2(self.gfx.size.0 as f32, self.gfx.size.1 as f32);
                        params.ssao = self.gfx.settings.ssao;
                        self.gfx.render_params.upload_to_gpu(&self.gfx.queue);

                        state
                            .light
                            .render_lights(&self.gfx, &mut enc, &sco, &lights);

                        self.gfx
                            .render_gui(&mut enc, &sco, |gctx| state.render_gui(window, gctx));
                        self.gfx.finish_frame(enc);

                        self.input.end_frame();
                        self.audio.update();
                    }
                },
                _ => (),
            }
        })
    }
}
