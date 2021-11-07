use std::borrow::Cow;
use std::path::PathBuf;
use std::time::SystemTime;

use wgpu::*;

pub struct ShaderLoader {
    module: Option<ShaderModule>,
    path: PathBuf,
    mtime: SystemTime,
}

impl ShaderLoader {
    pub fn new() -> Self {
        ShaderLoader {
            module: None,
            path: PathBuf::new(),
            mtime: SystemTime::UNIX_EPOCH,
        }
    }

    pub fn compile(device: &Device, source: &str) -> Option<ShaderModule> {
        // NOTE: a bit sad, wgpu-rs does not directly expose shader errors here :(
        // We have to do this until error scopes are implemented in wgpu-rs,
        // NOTE: https://github.com/gfx-rs/wgpu-rs/blob/3634abb0d560a2906d20c74efee9c2f16afb2503/src/backend/direct.rs#L818
        // NOTE: https://github.com/niklaskorz/linon/commit/63477a34110eca93bc7b70c97be91c262fca811b
        let (tx, rx) = crossbeam_channel::bounded(1);
        device.on_uncaptured_error(move |e: Error| {
            tx.send(e).unwrap();
        });

        let shader = device.create_shader_module(&ShaderModuleDescriptor {
            label: None,
            source: ShaderSource::Wgsl(Cow::Borrowed(source)),
        });

        device.on_uncaptured_error(move |e: Error| {
            panic!("{:#?}", e);
        });

        // try to revcieve an error
        match rx.try_recv() {
            // we failed to recieve the error,
            // so this is good actually
            Err(_) => Some(shader),

            // a compilation error occured :/
            Ok(e) => match e {
                Error::ValidationError {
                    description,
                    source,
                } => {
                    eprintln!("Shader Validation Error");
                    dbg!(source);
                    eprintln!("{}", description);
                    None
                }
                e => {
                    dbg!(e);
                    None
                }
            },
        }
    }

    pub fn load(&mut self, device: &Device, path: &str) -> (&ShaderModule, bool) {
        let path = PathBuf::from(path);
        let mtime = path.metadata().unwrap().modified().unwrap();

        if self.module.is_none() || mtime != self.mtime || self.path != path {
            let source = std::fs::read_to_string(&path).unwrap();
            self.mtime = mtime;
            self.path = path;

            match Self::compile(device, &source) {
                Some(module) => self.module = Some(module),
                None => (),
            };

            (self.module.as_ref().unwrap(), true)
        } else {
            (self.module.as_ref().unwrap(), false)
        }
    }
}
