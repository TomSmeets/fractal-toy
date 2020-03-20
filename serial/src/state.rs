use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs,
    fs::File,
    io::*,
    result::{Result, Result::*},
};

static DIR: &str = "target/state";

pub fn save<T: Serialize>(name: &str, m: &T) {
    fs::create_dir_all(DIR).unwrap();
    let file = File::create(format!("{}/{}.json", DIR, name)).unwrap();
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &m).unwrap();
}

pub fn load_in_place<T: DeserializeOwned>(name: &str, v: T) -> (T, Option<Error>) {
    let file = match File::open(format!("{}/{}.json", DIR, name)) {
        Ok(f) => f,
        Err(e) => return (v, Some(e)),
    };

    let mut reader = BufReader::new(file);

    drop(v);
    let v = serde_json::from_reader(&mut reader).unwrap();
    (v, None)
}

pub fn load<T: DeserializeOwned>(name: &str) -> Result<T, Error> {
    let file = File::open(format!("{}/{}.json", DIR, name))?;
    let mut reader = BufReader::new(file);
    let d1: T = serde_json::from_reader(&mut reader)?;
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
