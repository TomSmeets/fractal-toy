use wgpu::*;
use winit::window::Window;

use crate::tilemap::TilePos;
use crate::util::*;
use crate::viewport::Viewport;
use crate::Image;

mod pipeline;
mod swap_chain;

use self::pipeline::ShaderLoader;
use self::swap_chain::SwapChain;

// GPU mem = MAX_TILES * (vtx(5*4)*3*4 + 256*256)
const MAX_TILES: u32 = 512 * 2;
const MAX_VERTS: u64 = MAX_TILES as u64 * 3 * 4;

#[derive(Clone)]
pub struct TileSlot {
    id: u32,
    mode: SlotMode,
}

#[derive(Eq, PartialEq, Clone, Copy)]
pub enum SlotMode {
    // Free -> Used -> Old -> Free
    Free = 0,
    Used = 1,
    Old  = 2,
}

pub struct Gpu {
    device: Device,
    swap_chain: Option<SwapChain>,
    draw_tiles: Option<DrawTiles>,


    // move to draw_tiles
    shader: ShaderLoader,
    tile_count: u32,
    used: Vec<TileSlot>,
    vertex_list: Vec<Vertex>,
    // remove, upload directly
    upload_list: Vec<(u32, Image)>,
}

pub struct DrawTiles {
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,

    texture: Texture,
    sampler: Sampler,
    uniform: Buffer,

    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl DrawTiles {
    pub fn load(device: &mut Device, shader: &ShaderModule) -> DrawTiles {
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
                width: 256,
                height: 256,
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
        }
    }

    pub fn upload(&mut self, device: &mut Device, img: &Image, ix: u32) {
        device.queue.write_texture(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: ix,
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
    }
}

/// This struct should contain whatever the gpu should show
/// I don't like statefull apis, so this is the entire api
/// Put in here whatever you like, and the gpu will try to show it
pub struct GpuInput<'a> {
    pub resolution: V2<u32>,
    pub viewport: &'a Viewport,
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
pub struct UniformData {
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

pub struct Device {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: wgpu::Device,

    /// The queue is used to send commands to the gpu
    queue: Queue,

    swap_chain_format: TextureFormat,
}

impl Gpu {
    pub fn init(window: &Window) -> Gpu {
        // NOTE: does not have to be kept alive
        let instance = Instance::new(BackendBit::all());

        // surface and adapter
        let surface = unsafe { instance.create_surface(window) };

        // I don't want to deal with async stuff, so just block here.
        // In the far future we might want to support multiple adapters,
        // but I am not doing that now.
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
        }))
        .unwrap();

        // device, logical handle to the adapter.
        // TODO: setup, and figure out how tracing works.
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(), // TODO: add appropiate features here?
                limits: Limits::default(),   // TODO: also set to whaterver we are using?
            },
            None,
        ))
        .unwrap();

        let swap_chain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();

        Gpu {
            device: Device {
                surface,
                device,
                queue,
                swap_chain_format,
            },

            swap_chain: None,
            draw_tiles: None,
            shader: ShaderLoader::new(),

            used: vec![TileSlot { id: 0, mode: SlotMode::Free }; MAX_TILES as _],

            tile_count: 0,
            vertex_list: Vec::new(),
            upload_list: Vec::new(),
        }
    }

    #[rustfmt::skip]
    pub fn blit(&mut self, rect: &Rect, img: &Image) {
        let lx = rect.corner_min().x as f32;
        let ly = rect.corner_min().y as f32;
        let hx = rect.corner_max().x as f32;
        let hy = rect.corner_max().y as f32;

        // TODO: this is not good ofcourse
        let uv_x = img.size().x as f32 / 256.0;
        let uv_y = img.size().y as f32 / 256.0;

        let has_slot = self.used.iter_mut().enumerate().find(|(_, s)| s.id == img.id());

        let ix = match has_slot {
            Some((ix, slot)) => {
                // mark slot as still used
                slot.mode = SlotMode::Used;
                ix
            },

            None => {
                // find free slot
                let (ix, slot) = self.used.iter_mut().enumerate().find(|(_, s)| s.mode == SlotMode::Free).unwrap();
                self.upload_list.push((ix as u32, img.clone()));
                slot.id = img.id();
                slot.mode = SlotMode::Used;
                ix
            },
        };

        let ix = ix as i32;
        self.vertex_list.extend_from_slice(&[
            Vertex { pos: V2::new(lx, ly), uv: V2::new(0.0,  0.0),  ix, },
            Vertex { pos: V2::new(hx, ly), uv: V2::new(uv_x, 0.0),  ix, },
            Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0,  uv_y), ix, },
            Vertex { pos: V2::new(hx, ly), uv: V2::new(uv_x, 0.0),  ix, },
            Vertex { pos: V2::new(hx, hy), uv: V2::new(uv_x, uv_y), ix, },
            Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0,  uv_y), ix, },
        ]);
    }

    pub fn tile(&mut self, vp: &Viewport, p: &TilePos, img: &Image) {
        let rect = vp.world_to_screen_rect(&p.square());
        self.blit(&rect, img);
    }

    #[rustfmt::skip]
    pub fn render(&mut self, window: &Window, viewport: &Viewport) {
        let device = &mut self.device;

        let (swap_chain, frame) = loop {
            let swap_chain = self.swap_chain.get_or_insert_with(|| {
                let swap_chain = device.device.create_swap_chain(&device.surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: device.swap_chain_format,
                    width: viewport.size_in_pixels_i.x,
                    height: viewport.size_in_pixels_i.y,
                    present_mode: PresentMode::Mailbox,
                });

                SwapChain {
                    swap_chain,
                    resolution: viewport.size_in_pixels_i,
                }
            });

            if swap_chain.resolution != viewport.size_in_pixels_i {
                self.swap_chain = None;
                continue;
            }

            let frame = match swap_chain.swap_chain.get_current_frame() {
                Ok(frame) => frame,
                Err(e) => {
                    dbg!(e);
                    self.swap_chain = None;
                    continue;
                },
            };

            if frame.suboptimal {
                self.swap_chain = None;
                continue;
            }

            break (swap_chain, frame);
        };

        let (shader, shader_changed) = self.shader.load(&device.device, "src/gpu/shader.wgsl");

        if shader_changed {
            self.draw_tiles = None;
            // TODO: this is too late now, fix it
            for s in self.used.iter_mut() {
                s.mode = SlotMode::Free;
            }
        }

        let draw_tiles = self.draw_tiles.get_or_insert_with(|| DrawTiles::load(device, shader));

        // update slot age
        for s in self.used.iter_mut() {
            s.mode = match s.mode {
                SlotMode::Used => SlotMode::Old,
                SlotMode::Old  => SlotMode::Free,
                SlotMode::Free => SlotMode::Free,
            };
        }

        // upload new tile textures
        for (ix, img) in self.upload_list.drain(..) {
            eprintln!("upload: [{}] = {}", ix, img.id());
            draw_tiles.upload(device, &img, ix);
        }

        let vertex_list = &self.vertex_list[0..self.vertex_list.len().min(MAX_VERTS as usize)];

        // update uniform data
        device.queue.write_buffer(
            &draw_tiles.uniform,
            0,
            bytemuck::bytes_of(&UniformData {
                resolution: V2::new(
                    viewport.size_in_pixels.x as _,
                    viewport.size_in_pixels.y as _,
                ),
            }),
        );

        // write out vertex buffer
        device.queue.write_buffer(&draw_tiles.vertex_buffer, 0, bytemuck::cast_slice(&vertex_list));

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = device.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        // TODO: what do we do with compute commands? do they block? do we do them async?
        // Draw tiles
        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &frame.output.view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&draw_tiles.pipeline);
            rpass.set_vertex_buffer(0, draw_tiles.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &draw_tiles.bind_group, &[]);
            rpass.draw(0..vertex_list.len() as u32, 0..1);
        }

        // Draw ui with texture atlas
        {
        }

        device.queue.submit(Some(encoder.finish()));

        self.vertex_list.clear();
        self.tile_count = 0;
    }
}
