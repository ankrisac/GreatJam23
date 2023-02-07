use wgpu::util::DeviceExt;

use crate::graphics::Graphics;
use crate::nvec::*;

struct FontAtlas {
    bind_group: wgpu::BindGroup,
    dimensions: (u32, u32),
}
impl FontAtlas {
    const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

    fn bind_group_layout(gfx: &Graphics) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("GlyphRenderer.FontAtlas.BindGroupLayout"),
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

        let dimensions = image::GenericImageView::dimensions(&image);

        let extent = wgpu::Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture_with_data(
            &gfx.queue,
            &wgpu::TextureDescriptor {
                label: Some("GlyphRenderer.FontAtlas.Texture"),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: Self::FORMAT,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &image.to_rgba8(),
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("GlyphRenderer.FontAtlas.TextureView"),
            ..Default::default()
        });

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("GlyphRenderer.FontAtlas.Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("GlyphRenderer.FontAtlas.BindGroup"),
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
            bind_group,
            dimensions,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Glyph {
    pub pos: Vec3<f32>,
    pub codepoint: u32,
    pub scale: Vec2<f32>,
    pub color: Vec4<f32>
}

pub struct GlyphRenderer {
    pipeline: wgpu::RenderPipeline,

    atlas: FontAtlas,
    buffer: wgpu::Buffer,
}
impl GlyphRenderer {    
    const MAX_GLYPHS: usize = 1024;
    const GLYPH_ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3, 1 => Uint32, 2 => Float32x2, 3 => Float32x4
    ];
    const GLYPH_LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Glyph>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: Self::GLYPH_ATTRIBUTES,
    };


    pub fn new(gfx: &Graphics) -> Self {
        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("GlyphRenderer.Pipeline"),
                bind_group_layouts: &[&FontAtlas::bind_group_layout(gfx)],
                push_constant_ranges: &[],
            });

        let module = gfx.load_shader("shaders/glyph.wgsl");

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("GlyphRenderer.Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vert_main",
                    buffers: &[Self::GLYPH_LAYOUT],
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
                        format: gfx.get_format(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::OVER,
                            alpha: wgpu::BlendComponent::OVER,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("GlyphRenderer.GlyphBuffer"),
            size: (std::mem::size_of::<Glyph>() * Self::MAX_GLYPHS) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let atlas = FontAtlas::new(gfx, "assets/BasicFont.png");

        Self {
            pipeline,
            atlas,
            buffer
        }
    }

    pub fn render<'a>(&'a mut self, gfx: &Graphics, pass: &mut wgpu::RenderPass<'a>, glyphs: &[Glyph]) {
        if !glyphs.is_empty() {
            gfx.queue.write_buffer(
                &self.buffer,
                0,
                bytemuck::cast_slice(&glyphs),
            );    

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.atlas.bind_group, &[]);
            pass.set_vertex_buffer(0, self.buffer.slice(..));
            pass.draw(0..6, 0..glyphs.len() as u32);
        }
    }

    // Compute size of smallest font
    pub fn get_scale(&self, screen: (u32, u32)) -> Vec2<f32> {
        let px = 1.0 / screen.0 as f32;
        let py = 1.0 / screen.1 as f32;

        // Currently assuming an ASCII bitmap layout (16x8)
        let glyph_x = self.atlas.dimensions.0 as f32 / 16.0;
        let glyph_y = self.atlas.dimensions.1 as f32 / 8.0;

        vec2(px * glyph_x, py * glyph_y)
    }
}