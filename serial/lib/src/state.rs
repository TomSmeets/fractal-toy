use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs;
use std::fs::File;
use std::io::*;
use std::result::Result;
use std::result::Result::*;

static DIR: &str = "target/state";

pub fn save<T: Serialize>(name: &str, m: &T) {
	fs::create_dir_all(DIR).unwrap();
	let file = File::create(format!("{}/{}.json", DIR, name)).unwrap();
	let mut writer = BufWriter::new(file);
	serde_json::to_writer_pretty(&mut writer, &m).unwrap();
}

pub fn load<T: DeserializeOwned>(name: &str) -> Result<T, Error> {
	let file = File::open(format!("{}/{}.json", DIR, name))?;
	let mut reader = BufReader::new(file);
	let d1: T = serde_json::from_reader(&mut reader)?;
	return Ok(d1);
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
