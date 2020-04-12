use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Platform {
}

impl Platform {
    pub fn Platform() {
    }

    pub fn cpu_count(&self) -> i32 {
        sdl2::cpuinfo::cpu_count()
    }
}
