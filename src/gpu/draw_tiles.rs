use crate::gpu::*;
use crate::util::*;
use crate::viewport::Viewport;
use crate::Image;
use wgpu::*;

// GPU mem = MAX_TILES * (vtx(5*4)*3*4 + 256*256)
const MAX_TILES: u32 = 512 * 2;
const MAX_VERTS: u64 = MAX_TILES as u64 * 3 * 4;
const TILE_SIZE: u32 = 256;

pub struct DrawTiles {
    pub pipeline: RenderPipeline,
    pub vertex_buffer: Buffer,

    texture: Texture,
    sampler: Sampler,
    uniform: Buffer,

    bind_group_layout: BindGroupLayout,
    pub bind_group: BindGroup,

    used: Vec<TileSlot>,
    pub vertex_list: Vec<Vertex>,
}

impl DrawTiles {
    pub fn load(device: &mut GpuDevice, shader: &ShaderModule) -> DrawTiles {
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
                width: TILE_SIZE,
                height: TILE_SIZE,
                depth_or_array_layers: MAX_TILES,
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
                        view_dimension: TextureViewDimension::D2Array,
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

        DrawTiles {
            pipeline,
            vertex_buffer,

            uniform: uniform_buffer,

            texture,
            sampler,

            bind_group_layout,
            bind_group,

            used: vec![TileSlot { id: 0, mode: SlotMode::Free }; MAX_TILES as _],
            vertex_list: Vec::new(),
        }
    }

    pub fn blit(&mut self, device: &mut GpuDevice, rect: &Rect, img: &Image) {
        let lx = rect.corner_min().x as f32;
        let ly = rect.corner_min().y as f32;
        let hx = rect.corner_max().x as f32;
        let hy = rect.corner_max().y as f32;

        assert_eq!(img.size().x, TILE_SIZE);
        assert_eq!(img.size().y, TILE_SIZE);

        let has_slot = self
            .used
            .iter_mut()
            .enumerate()
            .find(|(_, s)| s.id == img.id());

        let ix = match has_slot {
            Some((ix, slot)) => {
                // mark slot as still used
                slot.mode = SlotMode::Used;
                ix
            },

            None => {
                // find free slot
                let (ix, slot) = self
                    .used
                    .iter_mut()
                    .enumerate()
                    .find(|(_, s)| s.mode == SlotMode::Free)
                    .unwrap();

                // mark slot as used
                slot.id = img.id();
                slot.mode = SlotMode::Used;

                // upload image
                eprintln!("upload: [{}] = {}", ix, img.id());
                device.queue.write_texture(
                    ImageCopyTexture {
                        texture: &self.texture,
                        mip_level: 0,
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: ix as u32,
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

                // return index for the vertex uv's
                ix
            },
        };

        let ix = ix as i32;

        if self.vertex_list.len() + 6 < MAX_VERTS as _ {
            self.vertex_list.extend_from_slice(&[
                Vertex { pos: V2::new(lx, ly), uv: V2::new(0.0, 0.0), ix, },
                Vertex { pos: V2::new(hx, ly), uv: V2::new(1.0, 0.0), ix, },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0, 1.0), ix, },

                Vertex { pos: V2::new(hx, ly), uv: V2::new(1.0, 0.0), ix, },
                Vertex { pos: V2::new(hx, hy), uv: V2::new(1.0, 1.0), ix, },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0, 1.0), ix, },
            ]);
        }
    }

    pub fn render(&mut self, device: &mut GpuDevice, viewport: &Viewport) {
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

        // update slot age
        for s in self.used.iter_mut() {
            s.mode = match s.mode {
                SlotMode::Used => SlotMode::Old,
                SlotMode::Old => SlotMode::Free,
                SlotMode::Free => SlotMode::Free,
            };
        }

        // clear last frame vertex list
        self.vertex_list.clear();
    }
}

#[derive(Clone)]
struct TileSlot {
    id: u32,
    mode: SlotMode,
}

#[derive(Eq, PartialEq, Clone, Copy)]
enum SlotMode {
    // Free -> Used -> Old -> Free
    Free = 0,
    Used = 1,
    Old = 2,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: V2<f32>,
    uv: V2<f32>,
    ix: i32,
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

impl Vertex {
    pub fn attrs() -> [VertexAttribute; 3] {
        vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
            2 => Sint32,
        ]
    }
}
