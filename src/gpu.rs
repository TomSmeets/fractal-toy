use cgmath::Vector2;
use wgpu::*;
use wgpu::util::*;
use winit::window::Window;

mod swap_chain;
mod pipeline;

use self::swap_chain::SwapChain;
use self::pipeline::Pipeline;

pub struct Gpu {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: Device,

    /// The queue is used to send commands to the gpu
    queue: Queue,

    // is only created as soon as we actually know what to draw
    swap_chain: SwapChain,
    pipeline: Pipeline,
}

/// This struct should contain whatever the gpu should show
/// I don't like statefull apis, so this is the entire api
/// Put in here whatever you like, and the gpu will try to show it
pub struct GpuInput {
    pub resolution: Vector2<u32>,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: Vector2<f32>,
}

unsafe impl bytemuck::Pod      for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}

impl Vertex {
    pub fn layout() -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as BufferAddress,
            step_mode: InputStepMode::Vertex,
            attributes: &vertex_attr_array![
                0 => Float32x2
            ],
        }
    }
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

        Gpu {
            surface,
            device,
            queue,
            swap_chain: SwapChain::new(swap_chain_format),
            pipeline: Pipeline::new("src/gpu/shader.wgsl"),
        }
    }

    pub fn render(&mut self, input: &GpuInput) {
        let frame = self.swap_chain.next_frame(&self.device, &self.surface, input.resolution);
        let pipeline = self.pipeline.load(&self.device, self.swap_chain.format());

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = self.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        let vertex_list = [
            Vertex { pos: Vector2::new(-1.0, -1.0) },
            Vertex { pos: Vector2::new( 1.0, -1.0) },
            Vertex { pos: Vector2::new(-1.0,  1.0) },

            Vertex { pos: Vector2::new( 1.0, -1.0) },
            Vertex { pos: Vector2::new( 1.0,  1.0) },
            Vertex { pos: Vector2::new(-1.0,  1.0) },

        ];
        
        let vertex_buffer = self.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&vertex_list),
            usage: BufferUsage::VERTEX,
        });

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
            rpass.set_vertex_buffer(0, vertex_buffer.slice(..));
            rpass.draw(0..vertex_list.len() as u32, 0..1);
        }

        self.queue.submit(Some(encoder.finish()));
    }
}
