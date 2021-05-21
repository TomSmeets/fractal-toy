use std::borrow::Cow;
use std::path::PathBuf;
use std::time::SystemTime;
use wgpu::*;

use crate::gpu::Vertex;

pub struct Pipeline {
    pipeline: Option<RenderPipeline>,
    path: PathBuf,
    mtime: SystemTime,
}

impl Pipeline {
    pub fn new(path: &str) -> Self {
        Pipeline {
            path: PathBuf::from(path),
            pipeline: None,
            mtime: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn is_wgsl_shader_valid(source: &str) -> bool {
        use naga::valid::ValidationFlags;
        use naga::valid::Validator;

        let module = match naga::front::wgsl::parse_str(&source) {
            Ok(m) => m,
            Err(e) => {
                dbg!(e);
                return false;
            },
        };

        // validate the IR
        let _ = match Validator::new(ValidationFlags::all()).validate(&module) {
            Ok(info) => Some(info),
            Err(error) => {
                dbg!(error);
                return false;
            },
        };

        return true;
    }

    pub fn load(&mut self, device: &Device, swap_chain_format: TextureFormat) -> &RenderPipeline {
        let mtime = self.path.metadata().unwrap().modified().unwrap();

        if self.pipeline.is_none() || mtime != self.mtime {
            println!("Recrating pipeline!");

            let source = std::fs::read_to_string(&self.path).unwrap();
            self.mtime = mtime;

            // NOTE: a bit sad, wgpu-rs does not directly expose shader errors here :(
            // Currently we just use naga to validate the shader here manually,
            // This is not very ideal, as wgpu-rs should just return a usefull Result type.
            // NOTE: https://github.com/gfx-rs/wgpu-rs/blob/3634abb0d560a2906d20c74efee9c2f16afb2503/src/backend/direct.rs#L818
            if Self::is_wgsl_shader_valid(&source) {
                let shader = device.create_shader_module(&ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Wgsl(Cow::Owned(source)),
                    flags: ShaderFlags::all(),
                });

                let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: None,
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

               let vertex_buffers = [VertexBufferLayout {
                    array_stride: (2*4) as BufferAddress,
                    step_mode: InputStepMode::Vertex,
                    attributes: &[
                        VertexAttribute {
                            format: VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                    ],
                }];

                let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
                    label: None,
                    layout: Some(&pipeline_layout),
                    vertex: VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::layout()],
                    },
                    fragment: Some(FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[swap_chain_format.into()],
                    }),
                    primitive: PrimitiveState::default(),
                    depth_stencil: None,
                    multisample: MultisampleState::default(),
                });

                self.pipeline = Some(pipeline);
            }
        }

        self.pipeline.as_ref().unwrap()
    }
}

