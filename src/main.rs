use winit::window::Window;

struct Graphics {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Graphics {
    async fn new(window: &Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            dx12_shader_compiler: Default::default(),
        });

        let surface = unsafe { instance.create_surface(window) }.expect("Unable to create Surface");

        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Inherit,
            view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
        };

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .expect("Unable to find GPU");

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Graphics"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .expect("Unable to connect to GPU");

        Self {
            surface,
            config,
            size,
            device,
            queue,
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width != 0 && new_size.height != 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    fn render(&mut self) {
        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(err) => {
                use wgpu::SurfaceError::*;
                match err {
                    Timeout => {},
                    Outdated | Lost => self.resize(self.size),
                    OutOfMemory => println!("Graphics: out of memory"),
                }

                return;
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Graphics.Encoder"),
            });

        {
            let time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();

            let r: f64 = 0.5 * time.sin() + 0.5;

            let pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Graphics.RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
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

    let mut gfx = pollster::block_on(Graphics::new(&window));

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

            WindowEvent::Resized(new_size) => gfx.resize(new_size),
            WindowEvent::ScaleFactorChanged { new_inner_size, .. } => gfx.resize(*new_inner_size),
            _ => {}
        },
        Event::MainEventsCleared => {
            gfx.render();
            window.request_redraw();
        }
        _ => {}
    })
}
