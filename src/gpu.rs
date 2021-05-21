use std::borrow::Cow;

use wgpu::*;
use winit::window::Window;
use cgmath::Vector2;

pub struct Gpu {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: Device,

    /// The queue is used to send commands to the gpu
    queue: Queue, 

    swap_chain_format: TextureFormat,

    // is only created as soon as we actually know what to draw
    swap_chain: Option<SwapChain>,
    pipeline:   Option<RenderPipeline>,
}

pub struct SwapChain {
    resolution: Vector2<u32>,
    swap_chain: wgpu::SwapChain,
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
            swap_chain_format,

            swap_chain: None,
            pipeline: None,
        }
    }

    pub fn render(&mut self, input: &GpuInput) {
        {
            let recreate_swapchain = match &self.swap_chain {
                None     => true,
                Some(sc) => sc.resolution != input.resolution,
            };

            if recreate_swapchain {
                println!("Recrating swapchain!");
                self.swap_chain = Some(SwapChain {
                    resolution: input.resolution,
                    swap_chain: self.device.create_swap_chain(&self.surface, &SwapChainDescriptor {
                        usage: TextureUsage::RENDER_ATTACHMENT,
                        format: self.swap_chain_format,
                        width:  input.resolution.x,
                        height: input.resolution.y,
                        present_mode: PresentMode::Mailbox,
                    }),
                });
            }
        }

        if self.pipeline.is_none() {
            println!("Recrating pipeline!");
            let shader = self.device.create_shader_module(&ShaderModuleDescriptor {
                label: None,
                source: ShaderSource::Wgsl(Cow::Owned(std::fs::read_to_string("src/shader.wgsl").unwrap())),
                flags: ShaderFlags::all(),
            });

            let pipeline_layout = self.device.create_pipeline_layout(&PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[],
                push_constant_ranges: &[],
            });

            let pipeline = self.device.create_render_pipeline(&RenderPipelineDescriptor {
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
                    targets: &[self.swap_chain_format.into()],
                }),
                primitive: PrimitiveState::default(),
                depth_stencil: None,
                multisample: MultisampleState::default(),
            });

            self.pipeline = Some(pipeline);
        }
        

        // TODO: is there a better way, instead of unwraping the swap_chain 2 times?
        let swap_chain = match &self.swap_chain {
            Some(sc) => sc,
            None => return,
        };

        let frame = match swap_chain.swap_chain.get_current_frame() {
            Ok(frame) => frame,
            Err(e) => {
                dbg!(e);

                // swap chain has to be recreated
                // lets drop this frame
                self.swap_chain = None;
                return;
            },
        };

        if frame.suboptimal {
            self.swap_chain = None;
            return;
        }

        let pipeline = match &self.pipeline {
            Some(pipe) => pipe,
            None => return,
        };

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
