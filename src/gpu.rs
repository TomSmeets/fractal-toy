use wgpu::*;
use winit::window::Window;

pub struct Gpu {
    /// The device is mostly used to allocate resources
    device: Device,

    /// The queue is used to send commands to the gpu
    queue: Queue, 
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

        Gpu {
            device,
            queue
        }
    }
}
