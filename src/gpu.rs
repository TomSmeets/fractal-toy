use std::borrow::Cow;
use std::path::PathBuf;
use std::time::SystemTime;

use wgpu::*;
use winit::window::Window;
use cgmath::Vector2;

pub struct Gpu {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: Device,

    /// The queue is used to send commands to the gpu
    queue: Queue, 

    // is only created as soon as we actually know what to draw
    swap_chain: SwapChain,
    pipeline:   Pipeline,
}

pub struct SwapChain {
    resolution: Vector2<u32>,
    format: TextureFormat,
    swap_chain: Option<wgpu::SwapChain>,
}

impl SwapChain {
    pub fn new(format: TextureFormat) -> Self {
        SwapChain {
            resolution: Vector2::new(0, 0),
            format,
            swap_chain: None,
        }
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }
    

    pub fn next_frame(&mut self, device: &Device, surface: &Surface, resolution: Vector2<u32>) -> SwapChainFrame {
        loop {
            let recreate_swapchain = self.swap_chain.is_none() || self.resolution != resolution;

            if recreate_swapchain {
                println!("Recrating swapchain!");
                self.resolution = resolution;
                self.swap_chain = Some(device.create_swap_chain(surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: self.format,
                    width:  resolution.x,
                    height: resolution.y,
                    present_mode: PresentMode::Mailbox,
                }));
            }

            let swap_chain = self.swap_chain.as_ref().unwrap();


            let frame = match swap_chain.get_current_frame() {
                Ok(frame) => frame,
                Err(e) => {
                    dbg!(e);

                    // swap chain has to be recreated
                    // lets drop this frame
                    self.swap_chain = None;
                    continue;
                },
            };

            if frame.suboptimal {
                self.swap_chain = None;
                    continue;
            }

            return frame;
        }
    }
}

struct Pipeline {
    pipeline: Option<RenderPipeline>,
    path: PathBuf,
    mtime: SystemTime,
}

impl Pipeline {
    pub fn new(path: &str) -> Self {
        Pipeline {
            path: PathBuf::from(path),
            pipeline: None,
            mtime: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn load(&mut self, device: &Device, swap_chain_format: TextureFormat) -> &RenderPipeline {
        let mtime = self.path.metadata().unwrap().modified().unwrap();

        if self.pipeline.is_none() || mtime != self.mtime {
            println!("Recrating pipeline!");

            let source = std::fs::read_to_string(&self.path).unwrap();
            self.mtime = mtime;

            let shader = device.create_shader_module(&ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(Cow::Owned(source)),
                flags: ShaderFlags::all(),
            });

            let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                label: None,
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
                },
                fragment: Some(FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    targets: &[swap_chain_format.into()],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
            });

            self.pipeline = Some(pipeline);
        }

        self.pipeline.as_ref().unwrap()
    }
}

/// This struct should contain whatever the gpu should show
/// I don't like statefull apis, so this is the entire api
/// Put in here whatever you like, and the gpu will try to show it
pub struct GpuInput {
    pub resolution: Vector2<u32>,
}

impl Gpu {
    pub fn init(window: &Window) -> Gpu {
        // choose whatever backend you want
        // NOTE: does not have to be kept alive
        let instance = Instance::new(BackendBit::all());

        // surface and adapter
        let surface = unsafe { instance.create_surface(window) };

        // I don't want to deal with async stuff, so just block here.
        // In the far future we might want to support multiple adapters,
        // but I am not doing that now.
        let adapter = pollster::block_on(instance.request_adapter(&  RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
        })).unwrap();

        // device, logical handle to the adapter.
        // TODO: setup, and figure out how tracing works. 
        let (device, queue) = pollster::block_on(adapter.request_device(&DeviceDescriptor {
            label: None,
            features: Features::empty(), // TODO: add appropiate features here?
            limits: Limits::default(), // TODO: also set to whaterver we are using?
        }, None)).unwrap();

        let swap_chain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();

        Gpu {
            surface,
            device,
            queue,
            swap_chain: SwapChain::new(swap_chain_format),
            pipeline: Pipeline::new("src/shader.wgsl"),
        }
    }

    pub fn render(&mut self, input: &GpuInput) {
        let frame = self.swap_chain.next_frame(&self.device, &self.surface, input.resolution);
        let pipeline = self.pipeline.load(&self.device, self.swap_chain.format());

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

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
            rpass.set_pipeline(&pipeline);
            rpass.draw(0..3, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
