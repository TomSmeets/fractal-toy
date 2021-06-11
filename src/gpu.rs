use wgpu::*;
use winit::window::Window;

use crate::util::*;
use crate::Image;
use crate::tilemap::TilePos;
use crate::viewport::Viewport;

mod swap_chain;
mod pipeline;

use self::swap_chain::SwapChain;
use self::pipeline::ShaderLoader;

// GPU mem = MAX_TILES * (vtx(5*4)*3*4 + 256*256)
const MAX_TILES: u32 = 512;
const MAX_VERTS: u64 = MAX_TILES as u64 * 3 * 4;

pub struct Gpu {
    device: Option<Device>,
    swap_chain: Option<SwapChain>,
    shader: ShaderLoader,
    other: Option<Other>,


    used: Vec<Option<TilePos>>,
    tile_count: u32,
    vertex_list: Vec<Vertex>,
    upload_list: Vec<(u32, Image)>,
}

pub struct Other {
    pipeline: RenderPipeline,

    vertex_buffer: Buffer,

    texture: Texture,
    sampler: Sampler,
    uniform: Buffer,

    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
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
    uv:  V2<f32>,
    ix: i32,
}

unsafe impl bytemuck::Pod      for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct UniformData {
    resolution: V2<f32>
}

unsafe impl bytemuck::Pod      for UniformData {}
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
    pub fn init() -> Gpu {
        Gpu {
            device: None,
            swap_chain: None,
            other: None,
            shader: ShaderLoader::new(),

            used: Vec::new(),

            tile_count: 0,
            vertex_list: Vec::new(),
            upload_list: Vec::new(),
        }
    }

    pub fn tile(&mut self, vp: &Viewport, p: &TilePos, img: &Image) {
        let ix = self.tile_count;
        self.tile_count += 1;

        while self.used.len() <= ix as usize {
            self.used.push(None);
        }

            let square = p.square();
            let low  = vp.world_to_screen(square.corner_min());
            let high = vp.world_to_screen(square.corner_max());

            let lx = low.x as f32;
            let ly = low.y as f32;
            let hx = high.x as f32;
            let hy = high.y as f32;
            
            // This is ofcourse very bad, but still bettern than nothing
            // TODO: improve, some kind of slotmap?
            if self.used[ix as usize] != Some(*p) {
                self.upload_list.push((ix, img.clone()));
                self.used[ix as usize] = Some(*p);
            }

            let ix = ix as i32;
            self.vertex_list.extend_from_slice(&[
                Vertex { pos: V2::new(lx, ly), uv: V2::new(0.0, 0.0), ix },
                Vertex { pos: V2::new(hx, ly), uv: V2::new(1.0, 0.0), ix },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0, 1.0), ix },

                Vertex { pos: V2::new(hx, ly), uv: V2::new(1.0, 0.0), ix },
                Vertex { pos: V2::new(hx, hy), uv: V2::new(1.0, 1.0), ix },
                Vertex { pos: V2::new(lx, hy), uv: V2::new(0.0, 1.0), ix },
            ]);
    }

    pub fn render(&mut self, window: &Window, input: &GpuInput) {
        let device = self.device.get_or_insert_with(|| {
            // choose whatever backend you want
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
            })).unwrap();

            // device, logical handle to the adapter.
            // TODO: setup, and figure out how tracing works.
            let (device, queue) = pollster::block_on(adapter.request_device(
                &DeviceDescriptor {
                    label: None,
                    features: Features::empty(), // TODO: add appropiate features here?
                    limits: Limits::default(),   // TODO: also set to whaterver we are using?
                },
                None,
            )).unwrap();

            let swap_chain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();

            Device {
                surface,
                device,
                queue,
                swap_chain_format,
            }
        });

        let (swap_chain, frame) = loop {
            let swap_chain = self.swap_chain.get_or_insert_with(|| {
                let swap_chain = device.device.create_swap_chain(&device.surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: device.swap_chain_format,
                    width: input.resolution.x,
                    height: input.resolution.y,
                    present_mode: PresentMode::Mailbox,
                });

                SwapChain { swap_chain, resolution: input.resolution }
            });

            if swap_chain.resolution != input.resolution {
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
            self.other = None;
            self.used.clear();
        }

        let other = self.other.get_or_insert_with(|| {
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
                format:  TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsage::COPY_DST | TextureUsage::SAMPLED,
                sample_count: 1,
                size: Extent3d { width: 256, height: 256, depth_or_array_layers: MAX_TILES },
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
                    }
                ],
            });

            let bind_group = device.device.create_bind_group(&BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry { binding: 0, resource: BindingResource::TextureView(&texture_view) },
                    BindGroupEntry { binding: 1, resource: BindingResource::Sampler(&sampler) },
                    BindGroupEntry { binding: 2, resource: uniform_buffer.as_entire_binding() }
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
                    targets: &[device.swap_chain_format.into()],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
            });

            Other {
                pipeline,
                vertex_buffer,

                uniform: uniform_buffer,

                texture,
                sampler,

                bind_group_layout,
                bind_group,
            }
        });

        for (ix, img) in self.upload_list.drain(..) {
            device.queue.write_texture(ImageCopyTexture {
                texture: &other.texture,
                mip_level: 0,
                origin: Origin3d {
                    x: 0,
                    y: 0,
                    z: ix as u32,
                },
            }, &img.data, ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(std::num::NonZeroU32::new(4*img.size.x).unwrap()),
                rows_per_image: Some(std::num::NonZeroU32::new(img.size.y).unwrap()),
            }, Extent3d {
                width: img.size.x,
                height: img.size.y,
                depth_or_array_layers: 1,
            });
        }

        let vertex_list = &self.vertex_list[0..self.vertex_list.len().min(MAX_VERTS as usize)];

        device.queue.write_buffer(&other.uniform, 0, bytemuck::bytes_of(&UniformData {
            resolution: V2::new(input.resolution.x as _, input.resolution.y as _),
        }));
        device.queue.write_buffer(&other.vertex_buffer, 0, bytemuck::cast_slice(&vertex_list));

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = device.device.create_command_encoder(&CommandEncoderDescriptor { label: None });


        // Render pass
        // TODO: what do we do with compute commands? do they block? do we do them async?
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
            rpass.set_pipeline(&other.pipeline);
            rpass.set_vertex_buffer(0, other.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &other.bind_group, &[]);
            rpass.draw(0..vertex_list.len() as u32, 0..1);
        }

        device.queue.submit(Some(encoder.finish()));

        dbg!(self.vertex_list.len());
        dbg!(self.tile_count);
        dbg!(self.upload_list.len());

        self.vertex_list.clear();
        self.tile_count = 0;
    }
}
