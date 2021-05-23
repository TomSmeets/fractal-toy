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

    pub fn load(&mut self, device: &Device, path: &str) -> (&ShaderModule, bool) {
        let path = PathBuf::from(path);
        let mtime = path.metadata().unwrap().modified().unwrap();

        if self.module.is_none() || mtime != self.mtime || self.path != path {
            let source = std::fs::read_to_string(&path).unwrap();
            self.mtime = mtime;
            self.path = path;

            // NOTE: a bit sad, wgpu-rs does not directly expose shader errors here :(
            // Currently we just use naga to validate the shader here manually,
            // This is not very ideal, as wgpu-rs should just return a usefull Result type.
            // NOTE: https://github.com/gfx-rs/wgpu-rs/blob/3634abb0d560a2906d20c74efee9c2f16afb2503/src/backend/direct.rs#L818
            if is_wgsl_shader_valid(&source) {
                let shader = device.create_shader_module(&ShaderModuleDescriptor {
                    label: None,
                    source: ShaderSource::Wgsl(Cow::Owned(source)),
                    flags: ShaderFlags::all(),
                });
                self.module = Some(shader);
            }
            (self.module.as_ref().unwrap(), true)
        } else {
            (self.module.as_ref().unwrap(), false)
        }
    }
}


fn is_wgsl_shader_valid(source: &str) -> bool {
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
