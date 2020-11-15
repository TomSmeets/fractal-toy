use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::fs::File;
use std::io::*;
use std::path::PathBuf;
use std::result::Result;

pub struct Persist {
    root: PathBuf,
}

impl Persist {
    pub fn new() -> Self {
        let root = PathBuf::from(".fractal-toy");
        fs::create_dir_all(&root).unwrap();
        Persist { root }
    }

    fn path_for(&self, name: &str) -> PathBuf {
        self.root.join(format!("{}.json", name))
    }

    pub fn save<T: Serialize>(&self, name: &str, m: &T) -> Result<(), Error> {
        let file = File::create(self.path_for(name))?;
        serde_json::to_writer_pretty(file, &m)?;
        Ok(())
    }

    pub fn load<T: DeserializeOwned>(&self, name: &str) -> Result<T, Error> {
        let file = File::open(self.path_for(name))?;
        let d1: T = serde_json::from_reader(file)?;
        Ok(d1)
    }

    pub fn list(&self) -> Vec<String> {
        let dir = fs::read_dir(&self.root);

        let dir = match dir {
            Ok(d) => d,
            Err(_) => return Vec::new(),
        };

        dir.map(|d| {
            d.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_os_string()
                .into_string()
                .unwrap()
        })
        .collect()
    }
}

pub trait Reload {
    type Storage;

    fn load(&mut self, data: Self::Storage);
    fn save(&self) -> Self::Storage;
}
