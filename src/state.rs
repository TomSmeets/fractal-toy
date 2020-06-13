use serde::{de::DeserializeOwned, Serialize};
use std::fs;
use std::fs::File;
use std::io::*;
use std::result::Result;

static DIR: &str = "target/state";

fn path_for(name: &str) -> String {
    format!("{}/{}.json", DIR, name)
}

pub fn save<T: Serialize>(name: &str, m: &T) {
    fs::create_dir_all(DIR).unwrap();
    let file = File::create(path_for(name)).unwrap();
    serde_json::to_writer_pretty(file, &m).unwrap();
}

pub fn load_in_place<T: DeserializeOwned>(name: &str, v: T) -> (T, Option<Error>) {
    let file = match File::open(path_for(name)) {
        Ok(f) => f,
        Err(e) => return (v, Some(e)),
    };

    let mut reader = BufReader::new(file);

    drop(v);
    let v = serde_json::from_reader(&mut reader).unwrap();
    (v, None)
}

pub fn load<T: DeserializeOwned>(name: &str) -> Result<T, Error> {
    let file = File::open(path_for(name))?;
    let d1: T = serde_json::from_reader(file)?;
    Ok(d1)
}

pub fn list() -> Vec<String> {
    let dir = fs::read_dir(DIR);

    match dir {
        Ok(d) => d
            .map(|d| {
                d.unwrap()
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap()
            })
            .collect(),
        Err(_) => Vec::new(),
    }
}
