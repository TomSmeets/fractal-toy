use cgmath::Vector2;

pub struct SwapChain {
    pub resolution: Vector2<u32>,
    pub swap_chain: wgpu::SwapChain,
}
