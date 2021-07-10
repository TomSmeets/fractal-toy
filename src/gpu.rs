use wgpu::*;
use winit::window::Window;

use crate::debug::Debug;
use crate::tilemap::TilePos;
use crate::util::*;
use crate::viewport::Viewport;
use crate::Image;
use crate::asset_loader::ImageID;

pub mod compute_tile;
mod draw_tiles;
mod draw_ui;
mod pipeline;
mod swap_chain;

use self::draw_tiles::DrawTiles;
use self::draw_ui::DrawUI;
use self::pipeline::ShaderLoader;
use self::swap_chain::SwapChain;
use crate::asset_loader::AssetLoader;
use std::sync::Arc;

pub struct Gpu {
    device: Arc<GpuDevice>,
    swap_chain: Option<SwapChain>,
    draw_tiles: DrawTiles,
    draw_ui: DrawUI,
}

/// This struct should contain whatever the gpu should show
/// I don't like statefull apis, so this is the entire api
/// Put in here whatever you like, and the gpu will try to show it
pub struct GpuInput<'a> {
    pub resolution: V2<u32>,
    pub viewport: &'a Viewport,
}

pub struct GpuDevice {
    surface: Surface,

    /// The device is mostly used to allocate resources
    device: wgpu::Device,

    /// The queue is used to send commands to the gpu
    queue: Queue,

    swap_chain_format: TextureFormat,
}

impl Gpu {
    pub fn init(window: &Window, asset_loader: &mut AssetLoader) -> Gpu {
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
        }))
        .unwrap();

        // device, logical handle to the adapter.
        // TODO: setup, and figure out how tracing works.
        let (device, queue) = pollster::block_on(adapter.request_device(
            &DeviceDescriptor {
                label: None,
                features: Features::empty(), // TODO: add appropiate features here? SHADER_FLOAT64 does not work yet correctly
                limits: Limits::default(),   // TODO: also set to whaterver we are using?
            },
            None,
        ))
        .unwrap();

        let swap_chain_format = adapter.get_swap_chain_preferred_format(&surface).unwrap();

        let device = GpuDevice {
            surface,
            device,
            queue,
            swap_chain_format,
        };

        let draw_ui = DrawUI::load(&device);
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

    pub fn blit(&mut self, asset_loader: &mut AssetLoader, rect: &Rect, img: ImageID) {
        self.draw_ui.blit(asset_loader, &self.device, rect, img)
    }

    pub fn tile(&mut self, vp: &Viewport, p: &TilePos, img: &Image) {
        let rect = vp.world_to_screen_rect(&p.square());
        self.draw_tiles.blit(&self.device, &rect, img);
    }

    #[rustfmt::skip]
    pub fn render(&mut self, window: &Window, viewport: &Viewport, debug: &mut Debug) {
        let device = &self.device;

        let (swap_chain, frame) = loop {
            let swap_chain = self.swap_chain.get_or_insert_with(|| {
                let swap_chain = device.device.create_swap_chain(&device.surface, &SwapChainDescriptor {
                    usage: TextureUsage::RENDER_ATTACHMENT,
                    format: device.swap_chain_format,
                    width: viewport.size_in_pixels_i.x,
                    height: viewport.size_in_pixels_i.y,
                    present_mode: PresentMode::Mailbox,
                });

                SwapChain {
                    swap_chain,
                    resolution: viewport.size_in_pixels_i,
                }
            });

            if swap_chain.resolution != viewport.size_in_pixels_i {
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

        debug.time("draw_tiles");
        let vtx_count = self.draw_tiles.vertex_list.len();
        self.draw_tiles.render(device, viewport);

        debug.time("draw_ui");
        let ui_vtx_count = self.draw_ui.vertex_list.len();
        self.draw_ui.render(device, viewport);

        // We finally have a frame, now it is time to create the render commands
        let mut encoder = device.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        // TODO: what do we do with compute commands? do they block? do we do them async?
        // How about instead of compute we just render to a texture view?
        // Draw tiles
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
        }

        debug.time("submit");
        device.queue.submit(Some(encoder.finish()));

    }
}
