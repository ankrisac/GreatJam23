mod graphics;
mod nvec;
mod sprite;
mod text;

use crate::graphics::*;
use crate::nvec::*;
use crate::sprite::*;

use winit::window::Window;



struct State {
    gfx: Graphics,
    text: text::TextRenderer,
}
impl State {
    async fn new(window: &Window) -> Self {
        let gfx = Graphics::new(window).await;
        let text = text::TextRenderer::new(&gfx);

        Self { gfx, text }
    }

    fn render(&mut self) -> Option<()> {
        let frame = self.gfx.new_frame()?;

        self.gfx.queue.submit([self.text.render(&self.gfx, &frame)]);
        frame.present();

        Some(())
    }

    fn update(&mut self) {
        
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
