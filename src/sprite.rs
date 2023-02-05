use wgpu::util::DeviceExt;

use crate::graphics::*;
use crate::nvec::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Sprite {
    pub pos: Vec3,
    pub color: Vec4,
    pub rot: f32,
}
impl Sprite {
    const ATTRIBUTES: &[wgpu::VertexAttribute] = &wgpu::vertex_attr_array![
        0 => Float32x3,
        1 => Float32x4,
        2 => Float32,
        3 => Float32
    ];

    const fn layout<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Sprite>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBUTES,
        }
    }
}

struct Atlas {
    texture: wgpu::Texture,

    bindgroup: wgpu::BindGroup,
}
impl Atlas {
    fn bind_group_layout(gfx: &Graphics) -> wgpu::BindGroupLayout {
        gfx.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Atlas.Texture"),
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

    fn new(gfx: &Graphics) -> Self {
        let diffuse_bytes =
            std::fs::read("assets/teapot.png").expect("unable to open [assets/teapot.png]");

        let diffuse_image = image::load_from_memory(diffuse_bytes.as_slice())
            .expect("unable to parse [assets/teapot.png]");

        let extent = wgpu::Extent3d {
            width: diffuse_image.width(),
            height: diffuse_image.height(),
            depth_or_array_layers: 1,
        };

        let texture = gfx.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Atlas.Texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        gfx.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_image.to_rgba8(),
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: std::num::NonZeroU32::new(4 * extent.width),
                rows_per_image: std::num::NonZeroU32::new(extent.height),
            },
            extent,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let sampler = gfx.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Atlas.Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bindgroup = gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Atlas.BindGroup"),
            layout: &&Self::bind_group_layout(gfx),
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

        Self { texture, bindgroup }
    }
}

pub struct SpriteRenderer {
    pipeline: wgpu::RenderPipeline,
    mesh_data: wgpu::Buffer,

    data: Vec<Sprite>,

    texture: Atlas,
}
impl SpriteRenderer {
    const MAX_SPRITES: wgpu::BufferAddress = 1024 * 1024;

    pub fn new(gfx: &Graphics) -> Self {
        let mut data = Vec::new();
        data.reserve_exact(Self::MAX_SPRITES as _);

        let mesh_data = gfx.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("SpriteRenderer.MeshData"),
            size: Self::MAX_SPRITES * std::mem::size_of::<Sprite>() as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("SpriteRenderer.Pipeline.Layout"),
                bind_group_layouts: &[&Atlas::bind_group_layout(gfx)],
                push_constant_ranges: &[],
            });

        let module = load_shader(gfx, "shaders/sprite.wgsl");

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("SpriteRenderer.Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: "vert_main",
                    buffers: &[Sprite::layout()],
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
                        format: wgpu::TextureFormat::Bgra8UnormSrgb,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                multiview: None,
            });

        let texture = Atlas::new(gfx);

        Self {
            pipeline,
            mesh_data,
            data,
            texture,
        }
    }
    pub fn render<'a>(&'a mut self, gfx: &Graphics, pass: &mut wgpu::RenderPass<'a>) {
        gfx.queue.write_buffer(
            &self.mesh_data,
            0,
            bytemuck::cast_slice(self.data.as_slice()),
        );

        pass.set_bind_group(0, &self.texture.bindgroup, &[]);
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.mesh_data.slice(..));
        pass.draw(0..6, 0..self.data.len() as u32);
    }

    pub fn draw(&mut self, sprite: Sprite) {
        if self.data.len() < Self::MAX_SPRITES as _ {
            self.data.push(sprite);
        } else {
            eprintln!("Skipping sprite: too many sprites");
        }
    }
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
