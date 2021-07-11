use crate::asset_loader::AssetLoader;
use crate::asset_loader::ImageID;
use crate::gpu::GpuDevice;
use crate::gpu::ShaderLoader;
use crate::pack::{Block, Pack};
use crate::util::*;
use crate::viewport::Viewport;
use std::collections::BTreeMap;
use wgpu::*;

const MAX_VERTS: u64 = 1024 * 4;
const ATLAS_SIZE: u32 = 1024;

pub struct DrawUI {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,

    texture: Texture,
    sampler: Sampler,
    uniform: Buffer,

    bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    pub vertex_list: Vec<Vertex>,

    blocks: BTreeMap<ImageID, (Block, V2<u32>)>,
    pack: Pack,
}

impl DrawUI {
    #[rustfmt::skip]
    pub fn load(device: &GpuDevice) -> Self {
        let mut loader = ShaderLoader::new();
        let (shader, _) = loader.load(&device.device, "src/gpu/draw_ui.wgsl");

        let vertex_buffer = device.device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Vertex>() as u64 * MAX_VERTS,
            mapped_at_creation: false,
            usage: BufferUsage::VERTEX | BufferUsage::COPY_DST,
        });

        // Uniform
        let uniform_buffer = device.device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<UniformData>() as u64,
            mapped_at_creation: false,
            usage: BufferUsage::UNIFORM | BufferUsage::COPY_DST,
        });

        // Texture
        let texture = device.device.create_texture(&TextureDescriptor {
            label: None,
            mip_level_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsage::COPY_DST | TextureUsage::SAMPLED,
            sample_count: 1,
            size: Extent3d {
                width: ATLAS_SIZE,
                height: ATLAS_SIZE,
                depth_or_array_layers: 1,
            },
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.device.create_sampler(&SamplerDescriptor {
            label: None,
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,

            mag_filter: FilterMode::Nearest,
            min_filter: FilterMode::Linear,

            mipmap_filter: FilterMode::Linear,
            ..SamplerDescriptor::default()
        });

        let bind_group_layout = device.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStage::FRAGMENT,
                    ty: BindingType::Sampler {
                        comparison: false,
                        filtering: true,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStage::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&texture_view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &Vertex::attrs(),
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: device.swap_chain_format,
                    blend: Some(BlendState {
                        color: BlendComponent::OVER,
                        alpha: BlendComponent::OVER,
                    }),
                    write_mask: ColorWrite::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        });

        DrawUI {
            pipeline,
            vertex_buffer,

            uniform: uniform_buffer,

            texture,
            sampler,

            bind_group_layout,
            bind_group,
            vertex_list: Vec::new(),

            blocks: BTreeMap::new(),
            pack: Pack::new(ATLAS_SIZE as _, 1),
        }
    }

    #[rustfmt::skip]
    pub fn blit(&mut self, asset_loader: &mut AssetLoader, device: &GpuDevice, rect: &Rect, img: ImageID) {
        let lx = rect.corner_min().x as f32;
        let ly = rect.corner_min().y as f32;
        let hx = rect.corner_max().x as f32;
        let hy = rect.corner_max().y as f32;

        // We don't free blocks yet, but we might in the future, just add a 'used' flag
        let blocks = &mut self.blocks;
        let pack = &mut self.pack;
        let texture = &self.texture;
        let (block, size) = blocks.entry(img).or_insert_with(|| {
            let img = asset_loader.get_image(img).unwrap();
            let size = img.size();
            let block = pack.alloc(size.map(|x| x as _)).unwrap();

            eprintln!("ui upload: {} = {:?}", img.id(), block);
            device.queue.write_texture(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: block.pos.x as u32,
                        y: block.pos.y as u32,
                        z: 0 as u32,
                    },
                },
                img.data(),
                ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(std::num::NonZeroU32::new(4 * img.size().x).unwrap()),
                    rows_per_image: Some(std::num::NonZeroU32::new(img.size().y).unwrap()),
                },
                Extent3d {
                    width: img.size().x,
                    height: img.size().y,
                    depth_or_array_layers: 1,
                },
            );

            (block, size)
        });

        let uv_lx = block.pos.x as f32 / ATLAS_SIZE as f32;
        let uv_ly = block.pos.y as f32 / ATLAS_SIZE as f32;
        let uv_hx = (block.pos.x + size.x as i32) as f32 / ATLAS_SIZE as f32;
        let uv_hy = (block.pos.y + size.y as i32) as f32 / ATLAS_SIZE as f32;

        if self.vertex_list.len() + 6 < MAX_VERTS as _ {
            self.vertex_list.extend_from_slice(&[
                Vertex { pos: V2::new(lx, ly), uv: V2::new(uv_lx, uv_ly),  },
                Vertex { pos: V2::new(hx, ly), uv: V2::new(uv_hx, uv_ly),  },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(uv_lx, uv_hy), },

                Vertex { pos: V2::new(hx, ly), uv: V2::new(uv_hx, uv_ly),  },
                Vertex { pos: V2::new(hx, hy), uv: V2::new(uv_hx, uv_hy), },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(uv_lx, uv_hy), },
            ]);
        } else {
            eprintln!("TOO MANY VERTS IN UI!");
        }
    }

    pub fn render(&mut self, device: &GpuDevice, viewport: &Viewport) {
        // update uniform data
        device.queue.write_buffer(
            &self.uniform,
            0,
            bytemuck::bytes_of(&UniformData {
                resolution: V2::new(
                    viewport.size_in_pixels.x as _,
                    viewport.size_in_pixels.y as _,
                ),
            }),
        );

        // write out vertex buffer
        device.queue.write_buffer(
            &self.vertex_buffer,
            0,
            bytemuck::cast_slice(&self.vertex_list),
        );

        // clear last frame vertex list
        self.vertex_list.clear();
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: V2<f32>,
    uv: V2<f32>,
}

impl Vertex {
    pub fn attrs() -> [VertexAttribute; 2] {
        vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ]
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct UniformData {
    resolution: V2<f32>,
}

unsafe impl bytemuck::Pod for UniformData {}
unsafe impl bytemuck::Zeroable for UniformData {}
