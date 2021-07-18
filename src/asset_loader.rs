use crate::glyph_cache::GlyphCache;
use crate::gpu::Gpu;
use crate::image::Image;
use crate::util::*;
use ::rusttype::Font;
use ::rusttype::Scale;
use std::collections::BTreeMap;
use std::time::SystemTime;

// OLD comment about ImageID's, but lets keep it here for now.
//
// I don't like this, but not sure what to do ye
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FontType {
    Normal,
    Mono,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextAlignment {
    Left = 0,
    Center = 1,
    Right = 2,
}

pub struct AssetLoader {
    // TODO: more font types
    //   mono    for debug text
    //   normral for ui ext
    // so two types should be enough
    font_mono: Font<'static>,
    font_norm: Font<'static>,

    glyph_cache_mono: GlyphCache,
    glyph_cache_norm: GlyphCache,

    image_cache: BTreeMap<String, (Image, SystemTime)>,
}

impl AssetLoader {
    pub fn new() -> Self {
        let font_mono = std::fs::read("./res/DejaVuSansMono.ttf").unwrap();
        let font_mono = Font::try_from_vec(font_mono).unwrap();

        let font_norm = std::fs::read("./res/DejaVuSans.ttf").unwrap();
        let font_norm = Font::try_from_vec(font_norm).unwrap();

        AssetLoader {
            image_cache: BTreeMap::new(),
            glyph_cache_norm: GlyphCache::new(),
            glyph_cache_mono: GlyphCache::new(),
            font_norm,
            font_mono,
        }
    }

    fn text_bounds(&self, font_type: FontType, scale: Scale, text: &str) -> V2<u32> {
        let font = match font_type {
            FontType::Mono => &self.font_mono,
            FontType::Normal => &self.font_norm,
        };

        let mut max_width = 0;
        let mut max_height = 0.0;
        let metrics = font.v_metrics(scale);
        let line_height = metrics.ascent - metrics.descent + metrics.line_gap;

        for line in text.lines() {
            max_height += line_height;
            if let Some(bb) = font
                .layout(line, scale, rusttype::Point { x: 0.0, y: 0.0 })
                .flat_map(|g| g.pixel_bounding_box())
                .last()
            {
                if bb.max.x > max_width {
                    max_width = bb.max.x;
                }
            }
        }

        V2::new(max_width as u32, max_height.ceil() as u32)
    }

    // TODO: This should not be here
    pub fn text(
        &mut self,
        font_type: FontType,
        p: V2<i32>,
        align: V2<TextAlignment>,
        size: f32,
        gpu: &mut Gpu,
        text: &str,
    ) {
        for (rect, img) in self.text_iter(font_type, p, align, size, text).into_iter() {
            gpu.blit(&rect, &img);
        }
    }

    // TODO: This should not be here
    pub fn text_iter(
        &mut self,
        font_type: FontType,
        p: V2<i32>,
        align: V2<TextAlignment>,
        size: f32,
        text: &str,
    ) -> Vec<(Rect, Image)> {
        let font_scale = Scale::uniform(size);

        let mut x = p.x as f32;
        let mut y = p.y as f32;

        if align.x != TextAlignment::Left && align.y != TextAlignment::Left {
            // align is an integer from 0 to 2
            let dx = (align.x as u32) as f32 * 0.5;
            let dy = (align.y as u32) as f32 * 0.5;

            // not ideal
            let bounds = self.text_bounds(font_type, font_scale, text);
            x -= bounds.x as f32 * dx;
            y -= bounds.y as f32 * dy;
        }

        let font = match font_type {
            FontType::Mono => &self.font_mono,
            FontType::Normal => &self.font_norm,
        };

        let cache = match font_type {
            FontType::Mono => &mut self.glyph_cache_mono,
            FontType::Normal => &mut self.glyph_cache_norm,
        };

        let metrics = font.v_metrics(font_scale);
        let line_height = metrics.ascent - metrics.descent + metrics.line_gap;

        let mut result = Vec::new();
        for line in text.lines() {
            y += line_height;
            let i = font.layout(line, font_scale, rusttype::Point { x, y });
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

                let img = cache.render_glyph(&g);
                result.push((rect, img.clone()));
            }
        }

        result
    }

    pub fn text_file(&mut self, path: &str) -> String {
        std::fs::read_to_string(path).unwrap()
    }

    pub fn hot_reload(&mut self) {
        let mut to_remove = Vec::new();

        for (path, (_, stored_mtime)) in self.image_cache.iter() {
            let current_mtime = std::fs::metadata(path).unwrap().modified().unwrap();

            if current_mtime != *stored_mtime {
                // yes thsese allocations are fine
                // this only happens very rarely, and during debugging.
                // And an empty Vec doesnt allocate
                to_remove.push(path.clone());
            }
        }

        // just remove them from the cache
        // this forces them to be reloaded the next time
        // Note that the images are reference counted, so they will not be deallocated yet
        // however no-one else should store them longer than a frame
        for path in to_remove {
            self.image_cache.remove(&path);
        }
    }

    pub fn image(&mut self, path: &str) -> Image {
        if let Some((img, _)) = self.image_cache.get(path) {
            return img.clone();
        }

        let img = loop {
            let buf = ::image::open(path);
            let buf = match buf {
                Ok(buf) => buf,
                Err(_) => {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                    continue;
                }
            };
            let buf = buf.into_rgba8();

            let (w, h) = buf.dimensions();
            let data = buf.into_raw();
            break Image::new(V2::new(w, h), data);
        };

        let mtime = std::fs::metadata(path).unwrap().modified().unwrap();
        self.image_cache
            .insert(path.to_string(), (img.clone(), mtime));

        return img;
    }
}
