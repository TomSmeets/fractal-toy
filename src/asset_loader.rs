use crate::*;
use rusttype::{Font, GlyphId, Scale};

pub struct AssetLoader {
    cache: BTreeMap<String, (SystemTime, Image)>,
    glyph_cache: BTreeMap<GlyphId, Image>,
    font: Font<'static>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let font = std::fs::read("result/share/fonts/truetype/DejaVuSansMono-Bold.ttf").unwrap();
        let font = Font::try_from_vec(font).unwrap();

        AssetLoader {
            cache: BTreeMap::new(),
            glyph_cache: BTreeMap::new(),
            font,
        }
    }

    pub fn text(&mut self, gpu: &mut Gpu, text: &str) {
        let mut y = 0.0;
        let font_scale = Scale::uniform(26.0);

        for line in text.lines() {
            let m = self.font.v_metrics(font_scale);
            y += m.ascent - m.descent + m.line_gap;
            let i = self
                .font
                .layout(line, font_scale, rusttype::Point { x: 0.0, y });
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

                let img = self.glyph_cache.entry(g.id()).or_insert_with(|| {
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

                gpu.blit(&rect, img);
            }
        }
    }

    pub fn text_file(&mut self, path: &str) -> String {
        std::fs::read_to_string(path).unwrap()
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
                    std::thread::sleep(std::time::Duration::from_millis(100));
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
