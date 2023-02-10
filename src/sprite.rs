use std::collections::HashMap;

use crate::graphics::Graphics;
use crate::nvec::*;

struct Atlas {
    texture: wgpu::Texture,
    bind_group: wgpu::BindGroup,
}
impl Atlas {
    fn layout(gfx: &Graphics) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("SpriteRenderer.Atlas.BindGroupLayout"),
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
        use wgpu::util::DeviceExt;

        let image_file = std::fs::read(path).expect(format!("Cannot read {path}").as_str());

        let image = image::load_from_memory(&image_file)
            .expect(format!("Could not parse file {path}").as_str());

        let extent = wgpu::Extent3d {
            width: image.width(),
            height: image.height(),
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture_with_data(
            &gfx.queue,
            &wgpu::TextureDescriptor {
                label: Some(format!("SpriteRenderer.Atlas[{path}].Texture").as_str()),
                size: extent,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            &image.to_rgba8(),
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some(format!("SpriteRenderer.Atlas[{path}].TextureView").as_str()),
            ..Default::default()
        });

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some(format!("SpriteRenderer.Atlas[{path}].Sampler").as_str()),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let bind_group = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(format!("SpriteRenderer.Atlas[{path}].BindGroup").as_str()),
            layout: &Self::layout(gfx),
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

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct UVRect {
    pub a: Vec2<f32>,
    pub b: Vec2<f32>,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Zeroable, bytemuck::Pod)]
pub struct Sprite {
    pub pos: Vec3<f32>,
    pub scale: Vec2<f32>,
    pub color: Vec4<f32>,
    pub rect: UVRect,
}

pub struct SpriteGroup {
    atlas: Atlas,
    data: Vec<Sprite>,
    buffer: wgpu::Buffer,
}
impl SpriteGroup {
    const MAX_SIZE: wgpu::BufferAddress = 128 * 1024;

    pub fn new(gfx: &Graphics, atlas_path: &str, instances: usize) -> Self {
        let atlas = Atlas::new(gfx, atlas_path);

        let data = Vec::new();
        let buffer = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SpriteGroup"),
            size: Self::MAX_SIZE * std::mem::size_of::<Sprite>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            atlas,
            data,
            buffer,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AtlasID(usize);

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
}
impl SpriteRenderer {
    pub fn new(gfx: &Graphics) -> Self {
        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SpriteRenderer.Pipeline.Layout"),
                bind_group_layouts: &[&Atlas::layout(gfx)],
                push_constant_ranges: &[],
            });

        let shader = gfx.load_shader("shaders/sprite.wgsl");

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SpriteRenderer.Pipeline"),
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
                        format: gfx.get_format(),
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        Self {
            pipeline,
        }
    }

    fn render<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>, groups: impl Iterator<Item = &'a SpriteGroup>) {
        pass.set_pipeline(&self.pipeline);
        for group in groups {
            pass.set_bind_group(0, &group.atlas.bind_group, &[]);
            pass.set_vertex_buffer(0, group.buffer.slice(..));
        }
    }
}
