use std::num::NonZeroU32;

use wgpu::*;

use crate::asset_loader::AssetLoader;
use crate::fractal::FractalStep;
use crate::gpu::GpuDevice;
use crate::gpu::ShaderLoader;
use crate::image::Image;
use crate::tilemap::TilePos;
use crate::util::*;

const TILE_SIZE: u32 = 256;

pub struct ComputeTile {
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,

    texture: Texture,
    texture_view: TextureView,
    buffer: Buffer,

    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
}

impl ComputeTile {
    pub fn load(alg: &[FractalStep], device: &GpuDevice, asset_loader: &mut AssetLoader) -> Self {
        let source = asset_loader.text_file("shader/compute_tile.wgsl");
        let source = source.replace("REAL", "f32");

        #[rustfmt::skip]
        let implementation = alg.iter().map(|x| match x {
            FractalStep::Conj   => "z.y = -z.y;\n",
            FractalStep::AbsR   => "z.x = abs(z.x);\n",
            FractalStep::AbsI   => "z.y = -abs(z.y);\n",
            FractalStep::Square => "z = cpx_sqr(z);\n",
            FractalStep::Cube   => "z = cpx_cube(z);\n",
            FractalStep::AddC   => "z = z + c;\nt = t + 1.0;\n",
        }).collect::<String>();

        let source = source.replace("@IMPL@", &implementation);

        let shader = ShaderLoader::compile(&device.device, &source).unwrap();

        let vertex_buffer = device.device.create_buffer(&BufferDescriptor {
            label: None,
            size: std::mem::size_of::<Vertex>() as u64 * 6,
            mapped_at_creation: false,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let copy_buffer = device.device.create_buffer(&BufferDescriptor {
            label: None,
            size: TILE_SIZE as u64 * TILE_SIZE as u64 * 4,
            mapped_at_creation: false,
            usage: BufferUsages::MAP_READ | BufferUsages::COPY_DST,
        });

        // Texture
        let texture = device.device.create_texture(&TextureDescriptor {
            label: None,
            mip_level_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            usage: TextureUsages::COPY_SRC | TextureUsages::RENDER_ATTACHMENT,
            sample_count: 1,
            size: Extent3d {
                width: TILE_SIZE,
                height: TILE_SIZE,
                depth_or_array_layers: 1,
            },
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        #[rustfmt::skip]
        let bind_group_layout = device.device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[],
        });

        let bind_group = device.device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[],
        });

        #[rustfmt::skip]
        let pipeline_layout = device.device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        #[rustfmt::skip]
        let pipeline = device.device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
                    step_mode: VertexStepMode::Vertex,
                    attributes: &Vertex::attrs(),
                }],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: device.swap_chain_format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
        });

        ComputeTile {
            pipeline,
            vertex_buffer,
            buffer: copy_buffer,
            texture,
            texture_view,
            bind_group_layout,
            bind_group,
        }
    }

    pub fn build(&self, device: &GpuDevice, p: &TilePos) -> Image {
        let rect = p.square();

        let min = rect.corner_min();
        let max = rect.corner_max();

        #[rustfmt::skip]
        let vertex_list = [
            Vertex { pos: V2::new(-1.0, -1.0), uv: V2::new(min.x as _, max.y as _), },
            Vertex { pos: V2::new( 1.0, -1.0), uv: V2::new(max.x as _, max.y as _), },
            Vertex { pos: V2::new(-1.0,  1.0), uv: V2::new(min.x as _, min.y as _), },

            Vertex { pos: V2::new( 1.0, -1.0), uv: V2::new(max.x as _, max.y as _), },
            Vertex { pos: V2::new( 1.0,  1.0), uv: V2::new(max.x as _, min.y as _), },
            Vertex { pos: V2::new(-1.0,  1.0), uv: V2::new(min.x as _, min.y as _), },
        ];

        // write out vertex buffer
        #[rustfmt::skip]
        device.queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertex_list));

        // We finally have a frame, now it is time to create the render commands
        #[rustfmt::skip]
        let mut encoder = device.device.create_command_encoder(&CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: None,
                color_attachments: &[RenderPassColorAttachment {
                    view: &self.texture_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });

            rpass.set_pipeline(&self.pipeline);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw(0..6, 0..1);
        }

        encoder.copy_texture_to_buffer(
            ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            ImageCopyBuffer {
                buffer: &self.buffer,
                layout: ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(NonZeroU32::new(4 * TILE_SIZE).unwrap()),
                    rows_per_image: Some(NonZeroU32::new(TILE_SIZE).unwrap()),
                },
            },
            Extent3d {
                width: TILE_SIZE,
                height: TILE_SIZE,
                depth_or_array_layers: 1,
            },
        );
        device.queue.submit(Some(encoder.finish()));

        let image = {
            let slice = self.buffer.slice(..);
            let fut = slice.map_async(MapMode::Read);
            device.device.poll(wgpu::Maintain::Wait);
            pollster::block_on(fut).unwrap();
            let bytes = slice.get_mapped_range();
            Image::new(V2::new(TILE_SIZE, TILE_SIZE), bytes.to_vec())
        };

        self.buffer.unmap();
        image
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vertex {
    pos: V2<f32>,
    uv: V2<f32>,
}

impl Vertex {
    pub fn attrs() -> [VertexAttribute; 2] {
        vertex_attr_array![
            0 => Float32x2,
            1 => Float32x2,
        ]
    }
}

unsafe impl bytemuck::Pod for Vertex {}
unsafe impl bytemuck::Zeroable for Vertex {}
