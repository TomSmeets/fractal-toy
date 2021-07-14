use crate::*;
use rusttype::{Font, GlyphId, Scale};

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct DataID(u32);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub enum ImageID {
    Raw(DataID),
    Glyph(GlyphId),
}

pub struct AssetLoader {
    cache: BTreeMap<String, (SystemTime, Image)>,
    glyph_cache: BTreeMap<GlyphId, Image>,
    font: Font<'static>,

    last_mtime: SystemTime,
    data_counter: DataID,
    data_cache: BTreeMap<String, DataID>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let font = std::fs::read("./res/DejaVuSansMono-Bold.ttf").unwrap();
        let font = Font::try_from_vec(font).unwrap();

        AssetLoader {
            cache: BTreeMap::new(),
            glyph_cache: BTreeMap::new(),
            font,

            last_mtime: SystemTime::now(),
            data_counter: DataID(1),
            data_cache: BTreeMap::new(),
        }
    }

    // TODO: This should not be here
    pub fn text(&mut self, gpu: &mut Gpu, text: &str) {
        let mut y = 0.0;
        let font_scale = Scale::uniform(26.0);

        for line in text.lines() {
            let m = self.font.v_metrics(font_scale);
            y += m.ascent - m.descent + m.line_gap;
            let i = self
                .font
                .layout(line, font_scale, rusttype::Point { x: 0.0, y });
            let mut blit = Vec::new();
            for g in i {
                let bb = match g.pixel_bounding_box() {
                    Some(bb) => bb,
                    None => continue,
                };
                let s = 1.0;
                let rect = Rect::min_max(
                    V2::new(bb.min.x as f64 * s, bb.min.y as f64 * s),
                    V2::new(bb.max.x as f64 * s, bb.max.y as f64 * s),
                );

                // insert glyph
                let _ = self.glyph_cache.entry(g.id()).or_insert_with(|| {
                    let mut data = vec![0; bb.width() as usize * bb.height() as usize * 4];
                    g.draw(|x, y, v| {
                        let ix = (y as usize * bb.width() as usize + x as usize) * 4;
                        let v = (v * 255.0).round() as u8;
                        data[ix + 0] = v;
                        data[ix + 1] = v;
                        data[ix + 2] = v;
                        data[ix + 3] = v;
                    });
                    Image::new(V2::new(bb.width() as _, bb.height() as _), data)
                });

                // return id
                blit.push((rect, ImageID::Glyph(g.id())));
            }

            for (rect, id) in blit.into_iter() {
                gpu.blit(self, &rect, id);
            }
        }
    }

    pub fn text_file(&mut self, path: &str) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    pub fn data(&mut self, path: &str) -> DataID {
        match self.data_cache.get(path).copied() {
            Some(id) => id,
            None => {
                let id = self.data_counter;
                self.data_counter.0 += 1;
                self.data_cache.insert(path.to_string(), id);
                id
            },
        }
    }

    pub fn hot_reload(&mut self) {
        let mut next_mtime = self.last_mtime;
        for (path, id) in self.data_cache.iter_mut() {
            let meta = std::fs::metadata(path).unwrap();
            let mtime = meta.modified().unwrap();
            if mtime > self.last_mtime {
                *id = self.data_counter;

                // new ids for those
                // we could also just remove them, but this might be faster
                self.data_counter.0 += 1;
                next_mtime = next_mtime.max(mtime);
            }
        }

        self.last_mtime = next_mtime;
    }

    pub fn image(&mut self, data: DataID) -> ImageID {
        ImageID::Raw(data)
    }

    pub fn get_path(&mut self, id: DataID) -> &str {
        let (path, _) = self.data_cache.iter().find(|(k, v)| **v == id).unwrap();
        path
    }

    pub fn get_data(&mut self, id: DataID) -> Vec<u8> {
        eprintln!("get_data({:?})", id);
        // slow, but reading a file is also slow, so this should be fine
        std::fs::read(self.get_path(id)).unwrap()
    }

    pub fn get_image(&mut self, id: ImageID) -> Option<Image> {
        eprintln!("get_image({:?})", id);

        match id {
            ImageID::Glyph(id) => self.glyph_cache.get(&id).cloned(),
            ImageID::Raw(id) => loop {
                let buf = ::image::open(self.get_path(id));
                let buf = match buf {
                    Ok(buf) => buf,
                    Err(_) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        continue;
                    },
                };
                let buf = buf.into_rgba8();

                let (w, h) = buf.dimensions();
                let data = buf.into_raw();
                let i = Image::new(V2::new(w, h), data);
                break Some(i);
            },
        }
    }

    /*
    pub fn get_regio(id: ImageID) -> Rect {
        match self.atlas.get(id) {
            Some(r) => r,
            None
    }
    */
}
