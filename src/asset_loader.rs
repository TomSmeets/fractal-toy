use crate::*;
use rusttype::{Font, GlyphId, PositionedGlyph, Scale};

// TODO: I don't like this, but not sure what to do yet
//
// All ui elements can be stored in memory, but they wouldent have to, as the atlas is
// permanent anyway. glyphs and ui images are never removed from the atlas
//
// The fractal tile images are removed regularly however, they also don't go in to the atlas,
// but in a array texture.
//
//
// Ok, I think i am going back to also storing everything on the cpu. because we are storing
// the biggest things, like the fractal tiles, on the cpu anyway. so no reason to  not do that
// with the ui elements, which are quite small.
//
// What about tiles generated on the gpu, can we somehow make them stay there?
//
// > Image(id, size, data: Gpu | Vec<u8>)
// could be a sloution
// the gpu texture array would be shared.
// > GpuMemory(atlas) // also remembers which slots are used
//
// All tiles are only stored in the builders. The gpu buffer is owned by the compute_tile
// builder. The gpu renderer should only use it.
//
// gpu.tile(rect, image) {
//   match image {
//      cpu => altas.alloc(imgage) -> use region
//      gpu => use buffer directly
//   }
// }
//
// This seems better, so not an image ID like this, just for fast equality checking. always
// include the data, because that is just easier. But the data can be stored on the gpu.
//
// Maybe maybe, we could even do
// > Image(id, size, data: Gpu | Vec<u8> | FileSystem(path))
// but don't know how it would be usefull
//
//
// Font renderer just has GlyphId -> Image like before

pub struct GlyphCache {
    // not using V2 here as it doesn't implement ord :/
    cache: BTreeMap<(GlyphId, [u16; 2], [u16; 2]), Image>,
}

impl GlyphCache {
    pub fn new() -> Self {
        GlyphCache {
            cache: BTreeMap::new(),
        }
    }

    pub fn render_glyph(&mut self, glyph: &PositionedGlyph) -> &Image {
        /// normally f.fract() can return negative numbers, because it rounds to 0,
        /// floor rounds to negative infinity, so this will alawys return a number form 0 to 1
        pub fn fract_abs(f: f32) -> f32 {
            f - f.floor()
        }

        let sub_pixel_steps = 16.0;

        let position = glyph.position();

        // sub-pixel position
        let sub_pixel_position = [
            (fract_abs(position.x) * sub_pixel_steps).floor() as u16,
            (fract_abs(position.y) * sub_pixel_steps).floor() as u16,
        ];

        let scale = glyph.scale();
        let sub_pixel_scale = [
            (scale.x * sub_pixel_steps).floor() as u16,
            (scale.y * sub_pixel_steps).floor() as u16,
        ];

        // TODO: The glyph id does not depend on the scale nor position!
        self.cache
            .entry((glyph.id(), sub_pixel_position, sub_pixel_scale))
            .or_insert_with(|| {
                let bb = match glyph.pixel_bounding_box() {
                    Some(bb) => bb,

                    // TODO: what is this? Just an empty glyph like a space?
                    None => rusttype::Rect {
                        min: rusttype::Point { x: 0, y: 0 },
                        max: rusttype::Point { x: 0, y: 0 },
                    },
                };

                let mut data = vec![0; bb.width() as usize * bb.height() as usize * 4];
                glyph.draw(|x, y, v| {
                    let ix = (y as usize * bb.width() as usize + x as usize) * 4;
                    let v = (v * 255.0).round() as u8;
                    data[ix + 0] = v;
                    data[ix + 1] = v;
                    data[ix + 2] = v;
                    data[ix + 3] = v;
                });

                Image::new(V2::new(bb.width() as _, bb.height() as _), data)
            })
    }
}

pub struct AssetLoader {
    font: Font<'static>,

    image_cache: BTreeMap<String, Image>,
    glyph_cache: GlyphCache,
}

impl AssetLoader {
    pub fn new() -> Self {
        let font = std::fs::read("./res/DejaVuSansMono-Bold.ttf").unwrap();
        let font = Font::try_from_vec(font).unwrap();

        AssetLoader {
            image_cache: BTreeMap::new(),
            glyph_cache: GlyphCache::new(),
            font,
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

                let img = self.glyph_cache.render_glyph(&g);

                // return id
                gpu.blit(&rect, img);
            }
        }
    }

    pub fn text_file(&mut self, path: &str) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    pub fn hot_reload(&mut self) {
        /*
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
        */
    }

    pub fn image(&mut self, path: &str) -> Image {
        if let Some(img) = self.image_cache.get(path) {
            return img.clone();
        }

        let img = loop {
            let buf = ::image::open(path);
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
            break Image::new(V2::new(w, h), data);
        };

        self.image_cache.insert(path.to_string(), img.clone());

        return img;
    }
}
