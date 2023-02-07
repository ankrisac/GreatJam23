use winit::window::Window;

pub struct Graphics {
    surface: wgpu::Surface,
    config: wgpu::SurfaceConfiguration,

    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl Graphics {
    pub async fn new(window: &Window) -> Self {
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
            view_formats: vec![],
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

                    // Note: remove later
                    features: wgpu::Features::POLYGON_MODE_LINE,
                    limits: wgpu::Limits::downlevel_webgl2_defaults(),
                },
                None,
            )
            .await
            .expect("Unable to connect to GPU");

        Self {
            surface,
            config,
            device,
            queue,
        }
    }

    pub fn get_size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }
    pub fn get_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width != 0 && new_size.height != 0 {
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn new_frame(&mut self) -> Option<Frame> {
        match self.surface.get_current_texture() {
            Ok(output) => {
                let view = output
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                Some(Frame { output, view })
            }
            Err(err) => {
                match err {
                    wgpu::SurfaceError::Timeout => {}
                    wgpu::SurfaceError::Outdated | wgpu::SurfaceError::Lost => {
                        let new_size =
                            winit::dpi::PhysicalSize::new(self.config.width, self.config.height);
                        self.resize(new_size);
                    }
                    wgpu::SurfaceError::OutOfMemory => println!("Graphics: out of memory"),
                }
                None
            }
        }
    }

    pub fn load_shader(&self, path: &str) -> wgpu::ShaderModule {
        let source =
            std::fs::read_to_string(path).expect(format!("unable to read file {path}").as_str());

        self.device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(path),
                source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(&source)),
            })
    }
}

pub struct Frame {
    output: wgpu::SurfaceTexture,
    pub view: wgpu::TextureView,
}
impl Frame {
    pub fn present(self) {
        self.output.present();
    }
}
