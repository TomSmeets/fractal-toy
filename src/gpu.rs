use std::sync::Arc;

use wgpu::*;
use winit::window::Window;

use self::draw_tiles::DrawTiles;
use self::draw_ui::DrawUI;
use self::pipeline::ShaderLoader;
use self::swap_chain::SwapChain;
use crate::asset_loader::AssetLoader;
use crate::debug::Debug;
use crate::image::Image;
use crate::tilemap::TilePos;
use crate::util::*;
use crate::viewport::Viewport;

pub mod compute_tile;
mod draw_tiles;
mod draw_ui;
mod pipeline;
mod swap_chain;

pub struct Gpu {
    device: Arc<GpuDevice>,
    swap_chain: Option<SwapChain>,
    draw_tiles: DrawTiles,
    draw_ui: DrawUI,
}

pub struct GpuDevice {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: wgpu::Device,

    /// The queue is used to send commands to the gpu
    queue: Queue,

    swap_chain_format: TextureFormat,
}

impl GpuDevice {
    pub fn init(window: &Window) -> Self {
        // NOTE: does not have to be kept alive
        let instance = Instance::new(Backends::VULKAN);

        // surface and adapter
        let surface = unsafe { instance.create_surface(window) };

        // I don't want to deal with async stuff, so just block here.
        // In the far future we might want to support multiple adapters,
        // but I am not doing that now.
        let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::default(),
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .unwrap();

        let mut limits = Limits::default();
        limits.max_texture_array_layers = 1024;

        // device, logical handle to the adapter.
        // TODO: setup, and figure out how tracing works.
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(), // TODO: add appropiate features here? SHADER_FLOAT64 does not work yet correctly
                limits,                      // TODO: also set to whaterver we are using?
            },
            None,
        ))
        .unwrap();

        let swap_chain_format = surface.get_preferred_format(&adapter).unwrap();
        dbg!(&swap_chain_format);

        GpuDevice {
            surface,
            device,
            queue,
            swap_chain_format,
        }
    }
}

impl Gpu {
    pub fn init(window: &Window, asset_loader: &mut AssetLoader) -> Gpu {
        let device = GpuDevice::init(window);
        let draw_ui = DrawUI::load(&device, asset_loader);
        let draw_tiles = DrawTiles::load(&device, asset_loader);

        Gpu {
            device: Arc::new(device),
            swap_chain: None,
            draw_tiles,
            draw_ui,
        }
    }

    pub fn device(&self) -> Arc<GpuDevice> {
        Arc::clone(&self.device)
    }

    pub fn blit(&mut self, rect: &Rect, img: &Image) {
        self.draw_ui.blit(&self.device, rect, img);
    }

    pub fn tile(&mut self, vp: &Viewport, p: &TilePos, img: &Image) {
        let rect = vp.world_to_screen_rect(&p.square());
        self.draw_tiles.blit(&self.device, &rect, img);
    }

    pub fn next_frame(&mut self, resolution: V2<u32>) -> (SurfaceTexture, TextureView) {
        loop {
            let need_resize = match &self.swap_chain {
                None => true,
                Some(sc) => sc.resolution != resolution,
            };

            if need_resize {
                self.device
                    .surface
                    .configure(&self.device.device, &SurfaceConfiguration {
                        usage: TextureUsages::RENDER_ATTACHMENT,
                        format: self.device.swap_chain_format,
                        width: resolution.x,
                        height: resolution.y,
                        present_mode: PresentMode::Mailbox,
                    });

                self.swap_chain = Some(SwapChain { resolution });
            }

            let frame = match self.device.surface.get_current_texture() {
                Ok(frame) => frame,
                Err(_) => {
                    self.swap_chain = None;
                    continue;
                }
            };

            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());
            break (frame, view);
        }
    }

    pub fn render(&mut self, window: &Window, resolution: V2<u32>) {
        let vtx_count = self.draw_tiles.vertex_list.len();
        self.draw_tiles
            .render(&self.device, resolution.map(|x| x as _));

        let ui_vtx_count = self
            .draw_ui
            .render(&self.device, resolution.map(|x| x as _));

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = self
            .device
            .device
            .create_command_encoder(&CommandEncoderDescriptor { label: None });

        let (frame, view) = self.next_frame(resolution);

        // TODO: what do we do with compute commands? do they block? do we do them async?
        // How about instead of compute we just render to a texture view?
        // Draw tiles
        {
            Debug::push("begin_render_pass");
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::RED),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            // rpass can be reused, but to what extend? multiple pipelines?
            rpass.set_pipeline(&self.draw_tiles.pipeline);
            rpass.set_vertex_buffer(0, self.draw_tiles.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.draw_tiles.bind_group, &[]);
            rpass.draw(0..vtx_count as u32, 0..1);

            // Draw ui with texture atlas
            rpass.set_pipeline(&self.draw_ui.pipeline);
            rpass.set_vertex_buffer(0, self.draw_ui.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.draw_ui.bind_group, &[]);
            rpass.draw(0..ui_vtx_count as u32, 0..1);
            Debug::pop();
        }

        Debug::push("submit");
        self.device.queue.submit(Some(encoder.finish()));
        Debug::pop();
        frame.present();
    }
}
