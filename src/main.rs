mod graphics;
mod nvec;
mod sprite;

use crate::graphics::*;
use crate::nvec::*;
use crate::sprite::*;

use winit::window::Window;

struct State {
    gfx: Graphics,
    draw: SpriteRenderer,

    begin: std::time::Instant,
}
impl State {
    async fn new(window: &Window) -> Self {
        let gfx = Graphics::new(window).await;
        let draw = SpriteRenderer::new(&gfx);

        let begin = std::time::Instant::now();

        Self { gfx, draw, begin }
    }

    fn render(&mut self) -> Option<()> {
        let frame = self.gfx.new_frame()?;

        let mut encoder = self
            .gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Graphics.Encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Graphics.RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.draw.render(&self.gfx, &mut pass);
        }

        self.gfx.queue.submit(std::iter::once(encoder.finish()));
        frame.output.present();

        Some(())
    }

    fn update(&mut self) {
        let time = std::time::Instant::now()
            .duration_since(self.begin)
            .as_secs_f32();

        self.draw.clear();

        fn wave(x: f32) -> f32 {
            0.5 + 0.4 * (0.5 * x).sin()
        }

        for i in -5..=5 {
            for j in -5..=5 {
                let x = i as f32;
                let y = j as f32;

                self.draw.draw(Sprite {
                    pos: vec3(x, y, 0.0),
                    color: vec4(wave(0.3 * x), wave(0.2 * y), 0.0, 1.0),
                    rot: 0.01 * x + 0.5 * y * time,
                });
            }
        }
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Hello Winit")
        .with_resizable(true)
        .with_transparent(true)
        .build(&event_loop)
        .expect("Unable to create window");

    let mut state = pollster::block_on(State::new(&window));

    use winit::event::{Event, KeyboardInput, VirtualKeyCode, WindowEvent};
    use winit::event_loop::ControlFlow;

    event_loop.run(move |event, _target, flow| match event {
        Event::WindowEvent { window_id, event } if window.id() == window_id => match event {
            WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => {
                *flow = ControlFlow::Exit;
            }

            WindowEvent::Resized(new_size) => state.gfx.resize(new_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                state.gfx.resize(*new_inner_size)
            }
            _ => {}
        },
        Event::MainEventsCleared => {
            state.update();
            state.render();
            window.request_redraw();
        }
        _ => {}
    })
}
