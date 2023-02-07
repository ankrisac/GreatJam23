use crate::graphics::{Frame, Graphics};

struct FontAtlas {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}
impl FontAtlas {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    fn bind_group_layout(gfx: &Graphics) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("FontAtlas.BindGroup"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            })
    }

    fn new(gfx: &Graphics, path: &str) -> Self {
        let image_file = std::fs::read(path).expect(format!("Could not read {path}").as_str());
        let image = image::load_from_memory(image_file.as_slice())
            .expect(format!("could not parse {path}").as_str());

        let extent = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("FontAtlas.Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gfx.queue.write_texture(
            wgpu::ImageCopyTextureBase {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * extent.width),
                rows_per_image: std::num::NonZeroU32::new(extent.height),
            },
            extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("FontAtlas.Texture.View"),
            ..Default::default()
        });

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("FontAtlas.Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("FontAtlas.BindGroup"),
            layout: &Self::bind_group_layout(gfx),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            texture,
            bind_group,
        }
    }
}

use crate::nvec::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Ascii(u32);

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct Glyph {
    pos: Vec4,
    scale: Vec2,
    codepoint: Ascii,
}
struct GlyphBuffer {
    buffer: wgpu::Buffer,
    data: Vec<Glyph>,
}
impl GlyphBuffer {
    const MAX_GLYPHS: usize = 1024;
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x4, 1 => Float32x2, 2 => Uint32
    ];
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Glyph>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::ATTRIBUTES,
    };

    fn new(gfx: &Graphics) -> Self {
        let mut data = Vec::new();
        data.reserve(Self::MAX_GLYPHS);

        let buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("TextRenderer.GlyphBuffer"),
            size: (std::mem::size_of::<Glyph>() * data.len()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self { buffer, data }
    }

    fn push_glyph(&mut self, glyph: Glyph) {
        if self.data.len() < Self::MAX_GLYPHS {
            self.data.push(glyph);
        } else {
            eprintln!("Glyph overflow");
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
struct ConfigData {
    scale: Vec2,
}

struct Config {
    buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    data: ConfigData,
}
impl Config {
    fn layout(gfx: &Graphics) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("TextRenderer.Config.BindGroupLayout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    fn new(gfx: &Graphics) -> Self {
        let data = ConfigData {
            scale: vec2(1.0, 1.0),
        };

        use wgpu::util::DeviceExt;
        let buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("TextRenderer.Config"),
                contents: bytemuck::bytes_of(&data),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_SRC,
            });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("TextRenderer.Config.BindGroup"),
            layout: &Self::layout(gfx),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &buffer,
                    offset: 0,
                    size: None,
                }),
            }],
        });

        Self {
            buffer,
            bind_group,
            data,
        }
    }
}

pub struct TextRenderer {
    pipeline: wgpu::RenderPipeline,

    atlas: FontAtlas,
    config: Config,
    text: GlyphBuffer,
}
impl TextRenderer {
    pub fn new(gfx: &Graphics) -> Self {
        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("TextureRenderer.Pipeline"),
                bind_group_layouts: &[&FontAtlas::bind_group_layout(gfx)],
                push_constant_ranges: &[],
            });

        let module = gfx.load_shader("shaders/text.wgsl");

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("TextRenderer.Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
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
                    module: &module,
                    entry_point: "frag_main",
                    targets: &[Some(wgpu::ColorTargetState {
                        format: gfx.config.format,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let atlas = FontAtlas::new(gfx, "assets/BasicFont.png");
        let text = GlyphBuffer::new(gfx);
        let config = Config::new(gfx);

        Self {
            pipeline,
            atlas,
            config,
            text,
        }
    }

    pub fn render(&self, gfx: &Graphics, frame: &Frame) -> wgpu::CommandBuffer {
        let mut encoder = gfx
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("TextRenderer.Encoder"),
            });
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("TextRenderer.RenderPass"),
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

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.atlas.bind_group, &[]);
            pass.set_bind_group(1, &self.config.bind_group, &[]);
            //pass.set_vertex_buffer(0, self.text.buffer.slice(..));
            pass.draw(0..6, 0..1 as u32);
        }

        encoder.finish()
    }
}
