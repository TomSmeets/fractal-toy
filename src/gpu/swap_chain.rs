use cgmath::Vector2;
use wgpu::*;

#[derive(Default)]
pub struct SwapChainC {
    pub inner: Option<SwapChain>,
}

impl SwapChainC {
    pub fn next_frame(
        &mut self,
        device: &Device,
        surface: &Surface,
        format: TextureFormat,
        resolution: Vector2<u32>,
    ) -> (SwapChain, SwapChainFrame) {
        if let Some(inner) = self.inner.take() {
            if inner.resolution == resolution {
                if let Ok(frame) = inner.swap_chain.get_current_frame() {
                    if !frame.suboptimal {
                        return (inner, frame);
                    }
                }
            }
        }

        self.inner = Some(SwapChain::new(device, surface, format, resolution));

        // if we did something wrong, this could loop, which is bad
        self.next_frame(device, surface,  format, resolution)
    }
}

pub struct SwapChain {
    pub resolution: Vector2<u32>,
    pub swap_chain: wgpu::SwapChain,
}

impl SwapChain {
    pub fn new(
        device: &Device,
        surface: &Surface,
        format: TextureFormat,
        resolution: Vector2<u32>,
    ) -> Self {
        let swap_chain = device.create_swap_chain(surface, &SwapChainDescriptor {
            usage: TextureUsage::RENDER_ATTACHMENT,
            format,
            width: resolution.x,
            height: resolution.y,
            present_mode: PresentMode::Mailbox,
        });

        SwapChain {
            resolution,
            swap_chain,
        }
    }
}
