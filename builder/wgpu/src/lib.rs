use wgpu::*;

pub struct Builder {
    adapter: Adapter,
    device: Device,
    queue: Queue,
}

impl Builder {
    pub async fn new() -> Self {
        let adapter = Adapter::request(
            &RequestAdapterOptions {
                power_preference: PowerPreference::Default,
                compatible_surface: None,
            },
            BackendBit::PRIMARY,
        )
        .await
        .unwrap();

        let (device, queue) = adapter.request_device(&DeviceDescriptor::default()).await;

        Builder {
            adapter,
            device,
            queue,
        }
    }
}
