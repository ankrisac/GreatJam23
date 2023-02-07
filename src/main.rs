mod glyph;
mod graphics;
mod gui;

mod input;

mod nvec;
mod sprite;

use crate::graphics::*;
use crate::nvec::*;
use crate::gui::Text;

use winit::window::Window;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PageState {
    MainMenu,
    Settings,
    Editor,
    Game,
    Exit,
}

struct Settings {
    fullscreen: bool,
}

struct App {
    window: Window,
    gfx: Graphics,
    glyph: glyph::GlyphRenderer,
    ui: gui::UserInterface,
    input: input::Input,

    page: PageState,

    settings: Settings,
}
impl App {
    async fn new(event_loop: &winit::event_loop::EventLoop<()>) -> Self {
        let settings = Settings { fullscreen: false };

        let window = winit::window::WindowBuilder::new()
            .with_title("Hello Winit")
            .with_min_inner_size(winit::dpi::PhysicalSize::new(600, 600))
            .with_resizable(true)
            .build(&event_loop)
            .expect("Unable to create window");

        let gfx = Graphics::new(&window).await;
        let glyph = glyph::GlyphRenderer::new(&gfx);
        let ui = gui::UserInterface::new();
        let input = input::Input::new();

        let page = PageState::MainMenu;

        Self {
            window,
            gfx,
            glyph,
            ui,
            input,
            page,
            settings,
        }
    }

    fn render(&mut self) -> Option<()> {
        let frame = self.gfx.new_frame()?;

        let mut encoder = self
            .gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("State.InitEncoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("State.RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            let cursor = crate::glyph::Glyph {
                pos: vec3(self.input.mouse.pos.x, self.input.mouse.pos.y, 0.0),
                codepoint: b'^' as u32,
                scale: self.ui.glyph_size,
                color: vec4(1.0, 0.0, 0.0, 1.0),
            };

            self.ui.glyphs.push(cursor);
            self.glyph.render(&self.gfx, &mut pass, &self.ui.glyphs);
        }

        self.gfx.queue.submit([encoder.finish()]);
        frame.present();

        Some(())
    }

    fn pager(&mut self) -> PageState {
        let fx = self.ui.glyph_unit.x;
        let fy = self.ui.glyph_unit.y;

        match self.page {
            PageState::MainMenu => {
                let left = f32::min(1.0, 52.0 * fx);
                let top = f32::min(1.0, 16.0 * fy);
                self.ui.anchor = vec2(-left, top);
                self.ui.set_fontsize(12.0);
                self.ui.label("Geomagika");

                self.ui.anchor.x += 10.0 * fx;
                self.ui.set_fontsize(5.0);
                if self.ui.button("New Game").clicked {
                    return PageState::Game;
                }
                if self.ui.button("Settings").clicked {
                    return PageState::Settings;
                }
                if self.ui.button("Editor").clicked {
                    return PageState::Editor;
                }
                if self.ui.button("Exit").clicked {
                    return PageState::Exit;
                }
            }
            PageState::Editor => {
                self.ui.anchor = vec2(0.0, 0.0);
                self.ui.set_fontsize(6.0);
                self.ui.label("Editor");

                self.ui.anchor = vec2(0.0, 4.0 * fy - 1.0);
                self.ui.set_fontsize(4.0);

                if self.ui.button("Back").clicked {
                    return PageState::MainMenu;
                }
            }
            PageState::Game => {
                self.ui.anchor = vec2(0.0, 0.0);
                self.ui.label("Game");

                let mut out = PageState::Game;
                if self.ui.button("Back").clicked {
                    out = PageState::Settings;
                }
                if self.ui.button("Back").clicked {
                    out = PageState::Editor;
                }
                return out;
            }
            PageState::Settings => {
                self.ui.anchor = vec2(0.0, 0.0);
                self.ui.set_fontsize(5.0);

                if self.ui.button("Fullscreen").clicked {
                    if !self.settings.fullscreen {
                        self.window
                            .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                    } else {
                        self.window.set_fullscreen(None);
                    }
                    self.settings.fullscreen = !self.settings.fullscreen;
                }
                if self.ui.button("Back").clicked {
                    return PageState::MainMenu;
                }
            }
            PageState::Exit => {}
        }
        return self.page;
    }

    fn update(&mut self) -> PageState {
        self.ui.glyphs.clear();
        self.ui.glyph_unit = self.glyph.get_scale(self.gfx.get_size());
        self.ui.mouse = self.input.mouse.clone();

        self.page = self.pager();
        self.page
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let mut app = pollster::block_on(App::new(&event_loop));

    use winit::event::{Event, WindowEvent};
    use winit::event_loop::ControlFlow;

    event_loop.run(move |event, _target, flow| match event {
        Event::WindowEvent { window_id, event } if app.window.id() == window_id => match event {
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            WindowEvent::Resized(new_size) => app.gfx.resize(new_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                app.gfx.resize(*new_inner_size)
            }
            WindowEvent::KeyboardInput { input, .. } => {}
            WindowEvent::CursorMoved { position, .. } => {
                let size = app.window.inner_size();

                let x = 2.0 * position.x / size.width as f64 - 1.0;
                let y = 1.0 - 2.0 * position.y / size.height as f64;

                app.input.mouse.set_pos(vec2(x as f32, y as f32));
            }
            WindowEvent::MouseInput { state, button, .. } => {
                app.input.mouse.set_state(state, button);
            }

            _ => {}
        },
        Event::MainEventsCleared => {
            if app.update() != PageState::Exit {
                app.render();
                app.window.request_redraw();
            }
            else {
                *flow = ControlFlow::Exit
            }
        }
        _ => {}
    })
}
