use winit::{
    event::{KeyboardInput, VirtualKeyCode},
    event_loop::ControlFlow,
};

struct Surface {
    handle: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
}
impl Surface {
    fn new(instance: &wgpu::Instance, window: &winit::window::Window) -> Self {
        let handle = unsafe { instance.create_surface(window) }.unwrap();
        let size = window.inner_size();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8UnormSrgb,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![wgpu::TextureFormat::Bgra8UnormSrgb],
        };

        Self {
            handle,
            config,
            size,
        }
    }
    fn resize(&mut self, device: &wgpu::Device, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.handle.configure(device, &self.config);
    }
}

struct Context {
    surface: Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl Context {
    async fn new(window: &winit::window::Window) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        let surface = Surface::new(&instance, &window);

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface.handle),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Context"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .unwrap();

        Self {
            surface,
            device,
            queue,
        }
    }
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.surface.resize(&self.device, new_size);
    }
    fn render(&mut self) {

    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Hello winit")
        .build(&event_loop)
        .expect("Failed to create window");

    let mut context = pollster::block_on(Context::new(&window));

    event_loop.run(move |event, _, flow| match event {
        winit::event::Event::WindowEvent { window_id, event } => {
            if window_id == window.id() {
                use winit::event::WindowEvent;

                match event {
                    WindowEvent::CloseRequested => {
                        *flow = ControlFlow::Exit;
                    }
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
                    WindowEvent::Resized(new_size) => context.resize(new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        context.resize(*new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        winit::event::Event::MainEventsCleared => {}
        _ => {}
    });
}
