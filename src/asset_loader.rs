use std::borrow::Cow;
use std::collections::BTreeMap;
use std::time::SystemTime;

use ::rusttype::Font;
use ::rusttype::Scale;

use crate::debug::Debug;
use crate::glyph_cache::GlyphCache;
use crate::gpu::Gpu;
use crate::image::Image;
use crate::util::*;

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

pub struct Data {
    id: u32,
}

include!(concat!(env!("OUT_DIR"), "/static_res_files.rs"));

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
        let font_mono = STATIC_RES_FILES.get("res/DejaVuSansMono.ttf").unwrap();
        let font_mono = font_from_cow(font_mono);

        let font_norm = STATIC_RES_FILES.get("res/DejaVuSans.ttf").unwrap();
        let font_norm = font_from_cow(font_norm);

        AssetLoader {
            image_cache: BTreeMap::new(),
            glyph_cache_norm: GlyphCache::new(),
            glyph_cache_mono: GlyphCache::new(),
            font_norm,
            font_mono,
        }
    }

    pub fn text_bounds(&self, font_type: FontType, scale: Scale, text: &str) -> Rect {
        #[rustfmt::skip]
        let font = match font_type {
            FontType::Mono   => &self.font_mono,
            FontType::Normal => &self.font_norm,
        };

        let mut y = 0.0;
        let metrics = font.v_metrics(scale);
        let line_height = metrics.ascent - metrics.descent + metrics.line_gap;

        let mut min = V2::zero();
        let mut max = V2::zero();
        let mut first = true;
        for line in text.lines() {
            y += line_height;
            for bb in font
                .layout(line, scale, rusttype::Point { x: 0.0, y })
                .flat_map(|g| g.pixel_bounding_box())
            {
                let bb_min = V2::new(bb.min.x as f64, bb.min.y as f64);
                let bb_max = V2::new(bb.max.x as f64, bb.max.y as f64);

                if first {
                    min = bb_min;
                    max = bb_max;
                    first = false;
                } else {
                    min.x = min.x.min(bb_min.x);
                    min.y = min.y.min(bb_min.y);
                    max.x = max.x.max(bb_max.x);
                    max.y = max.y.max(bb_max.y);
                }
            }
        }

        Rect::min_max(min, max)
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

        let bounds = self.text_bounds(font_type, font_scale, text);

        match align.x {
            TextAlignment::Left   => x -= bounds.corner_min().x as f32,
            TextAlignment::Center => x -= bounds.center().x     as f32,
            TextAlignment::Right  => x -= bounds.corner_max().x as f32,
        }

        match align.y {
            TextAlignment::Left   => y -= bounds.corner_min().y as f32,
            TextAlignment::Center => y -= bounds.center().y     as f32,
            TextAlignment::Right  => y -= bounds.corner_max().y as f32,
        }

        let font = match font_type {
            FontType::Mono   => &self.font_mono,
            FontType::Normal => &self.font_norm,
        };

        let cache = match font_type {
            FontType::Mono   => &mut self.glyph_cache_mono,
            FontType::Normal => &mut self.glyph_cache_norm,
        };

        let metrics = font.v_metrics(font_scale);
        let line_height = metrics.ascent - metrics.descent + metrics.line_gap;

        let mut result = Vec::new();
        Debug::push("AssetLoader.text_iter");
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
        Debug::pop();

        result
    }

    pub fn text_file(&mut self, path: &str) -> String {
        String::from_utf8(STATIC_RES_FILES.get(path).unwrap().into_owned()).unwrap()
    }

    pub fn hot_reload(&mut self) {
        if true {
            return;
        }

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
            let data = STATIC_RES_FILES.get(path).unwrap();
            let buf = ::image::load_from_memory(&data);
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

        self.image_cache
            .insert(path.to_string(), (img.clone(), SystemTime::now()));
        return img;
    }
}

fn font_from_cow(data: Cow<'static, [u8]>) -> Font<'static> {
    match data {
        Cow::Owned(data) => Font::try_from_vec(data).unwrap(),
        Cow::Borrowed(data) => Font::try_from_bytes(data).unwrap(),
    }
}
