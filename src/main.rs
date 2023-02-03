use std::borrow::Cow::Borrowed;

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

struct Graphics {
    surface: Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
}
impl Graphics {
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
    fn get_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        use wgpu::SurfaceError::*;

        match self.surface.handle.get_current_texture() {
            Ok(frame) => return Some(frame),
            Err(err) => match err {
                Timeout => {}
                Outdated | Lost => self.surface.resize(&self.device, self.surface.size),
                OutOfMemory => panic!("Error out of memory"),
            },
        }

        None
    }

    fn load_shader(&self, path: &str) -> wgpu::ShaderModule {
        let source = std::fs::read_to_string(path).expect("unable to read file");

        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(path),
                source: wgpu::ShaderSource::Wgsl(Borrowed(source.as_str())),
            })
    }
}

struct Blitz {
    pipeline: wgpu::RenderPipeline,
}
impl Blitz {
    const LABEL: Option<&str> = Some("Blitz");

    fn new(gfx: &Graphics) -> Self {
        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Self::LABEL,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

        let shader = gfx.load_shader("shaders/blitz.wgsl");

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Self::LABEL,
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vert_main",
                    buffers: &[],
                },
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "frag_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self { pipeline }
    }

    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}

struct State {
    gfx: Graphics,
    blitz: Blitz,
}
impl State {
    fn new(gfx: Graphics) -> Self {
        let blitz = Blitz::new(&gfx);

        Self { gfx, blitz }
    }
    fn render(&mut self) -> Option<()> {
        let frame = self.gfx.get_frame()?;

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("state"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("state"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

            self.blitz.render(&mut pass);
        }
        self.gfx.queue.submit(std::iter::once(encoder.finish()));

        frame.present();

        Some(())
    }
}

fn main() {
    let event_loop = winit::event_loop::EventLoop::new();

    let window = winit::window::WindowBuilder::new()
        .with_title("Hello winit")
        .build(&event_loop)
        .expect("Failed to create window");

    let context = pollster::block_on(Graphics::new(&window));
    let mut state = State::new(context);

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
                    WindowEvent::Resized(new_size) => state.gfx.resize(new_size),
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        state.gfx.resize(*new_inner_size)
                    }
                    _ => {}
                }
            }
        }
        winit::event::Event::MainEventsCleared => {
            state.render();
        }
        _ => {}
    });
}
