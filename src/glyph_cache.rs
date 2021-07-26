use std::collections::BTreeMap;

use ::rusttype::GlyphId;
use ::rusttype::PositionedGlyph;

use crate::debug::Debug;
use crate::image::Image;
use crate::util::V2;

/// Cache glyphs of a given font
///
/// TODO: make work with multiple fonts, glyphid is unique per font
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
        Debug::push("GlyphCache.render_glyph");

        /// normally f.fract() can return negative numbers, because it rounds to 0,
        /// floor rounds to negative infinity, so this will alawys return a number form 0 to 1
        pub fn fract_abs(f: f32) -> f32 {
            f - f.floor()
        }

        // TODO: the cache could explode if we choose this too big
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
        let result = self
            .cache
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
            });
        Debug::pop();
        result
    }
}
