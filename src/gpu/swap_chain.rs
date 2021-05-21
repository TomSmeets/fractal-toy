use cgmath::Vector2;
use wgpu::*;

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

    pub fn next_frame(
        &mut self,
        device: &Device,
        surface: &Surface,
        resolution: Vector2<u32>,
    ) -> SwapChainFrame {
        loop {
            let recreate_swapchain = self.swap_chain.is_none() || self.resolution != resolution;

            if recreate_swapchain {
                println!("Recrating swapchain!");
                self.resolution = resolution;
                self.swap_chain = Some(device.create_swap_chain(surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: self.format,
                    width: resolution.x,
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

