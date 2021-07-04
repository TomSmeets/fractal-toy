use crate::*;

pub struct AssetLoader {
    cache: BTreeMap<String, (SystemTime, Image)>,
}

impl AssetLoader {
    pub fn new() -> Self {
        AssetLoader {
            cache: BTreeMap::new(),
        }
    }

    pub fn image(&mut self, path: &str) -> Image {
        let time = std::fs::metadata(path).unwrap().modified().unwrap();

        if let Some((t, i)) = self.cache.get(path) {
            if t == &time {
                return i.clone();
            }
        }

        loop {
            let buf = ::image::open(path);
            let buf = match buf {
                Err(e) => {
                    dbg!(e);
                    std::thread::sleep_ms(100);
                    continue;
                },
                Ok(buf) => buf,
            };
            let buf = buf.into_rgba8();

            let (w, h) = buf.dimensions();
            let data = buf.into_raw();
            let i = Image::new(V2::new(w, h), data);
            self.cache.insert(path.to_string(), (time, i.clone()));
            break i;
        }
    }
}
