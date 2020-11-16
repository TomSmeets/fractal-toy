mod atlas;
mod input;

use self::atlas::Atlas;
use self::input::SDLInput;

use crate::math::*;
use crate::AtlasRegion;
use crate::Config;
use crate::Input;
use crate::Tile;
use crate::TileMap;
use crate::TileParams;
use crate::Viewport;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use std::collections::BTreeMap;
use tilemap::CompareIter;
use tilemap::ComparedValue;

pub fn rect_to_sdl(r: Rect) -> sdl2::rect::Rect {
    sdl2::rect::Rect::new(r.pos.x, r.pos.y, r.size.x as u32, r.size.y as u32)
}

pub struct Sdl {
    /// ~~SDL_Quit is called when dropped, so it has to be kept alive~~
    /// Never mind, that is not true, the context is only dropped when all SDL
    /// elements are dropped. So it is not necessary to keep the context or
    /// subsystem in memory. I will however keep these fields. as to make it
    /// explicit that we are using this.
    pub ctx: sdl2::Sdl,
    pub video: sdl2::VideoSubsystem,
    pub controller: sdl2::GameControllerSubsystem,
    pub event: sdl2::EventPump,
    pub canvas: Canvas<Window>,

    pub input: SDLInput,
    pub map: tilemap::TileMap<AtlasRegion>,
    pub atlas: Atlas,
    pub version: u32,
}

impl Sdl {
    pub fn new(config: &Config) -> Self {
        let ctx = sdl2::init().unwrap();
        let video = ctx.video().unwrap();

        let window = video
            .window("rust-sdl2 demo", 800, 600)
            .resizable()
            .opengl()
            .position_centered()
            .build()
            .unwrap();

        // IMPOTANT: This causes some issues in older sdl2 versions
        // For some reason this causes horrible lag spikes when polling events
        // see https://stackoverflow.com/a/53658644
        // however i have also noticed the same issue when on version '355a4f94a782' of sdl2
        // not sure which commit fixed it, but with the stable sdl-2.0.12 it seems to work
        let controller = ctx.game_controller().unwrap();

        let event = ctx.event_pump().unwrap();
        let mut canvas = window.into_canvas().present_vsync().build().unwrap();

        canvas.set_blend_mode(BlendMode::Blend);

        unsafe {
            sdl2::sys::SDL_SetHint(
                sdl2::sys::SDL_HINT_RENDER_SCALE_QUALITY.as_ptr() as *const i8,
                (b"0").as_ptr() as *const i8,
            );
        }

        Sdl {
            ctx,
            video,
            controller,
            event,
            canvas,

            version: 0,

            input: SDLInput::new(),
            map: tilemap::TileMap::new(),
            atlas: Atlas::new(config.params.size),
        }
    }

    pub fn events(&mut self) -> Input {
        let es = self.event.poll_iter().collect::<Vec<_>>();
        let input = self.input.handle_sdl(&es);
        input
    }

    pub fn render(&mut self, params: &TileParams, map: &TileMap, vp: &Viewport) {
        if params.version != self.version {
            self.map.clear();
            self.atlas.clear();
            self.version = params.version;
        }

        // Update my renderd tiles
        let tiles = std::mem::replace(&mut self.map.tiles, BTreeMap::new());
        let iter = CompareIter::new(map.tiles.iter(), tiles.into_iter(), |(l, _), (r, _)| {
            l.cmp(&r)
        });

        for i in iter {
            match i {
                ComparedValue::Left((pos, v)) => {
                    // only in map.tiles
                    if let Tile::Done(px) = v {
                        let texture_creator = self.canvas.texture_creator();
                        let region = self.atlas.alloc(&texture_creator, px);
                        self.map.tiles.insert(*pos, region);
                    }
                },

                ComparedValue::Right((_, region)) => {
                    // only in sdl
                    self.atlas.remove(region);
                },

                ComparedValue::Both((pos, _), (_, w)) => {
                    self.map.tiles.insert(*pos, w);
                },
            }
        }

        // Clear canvas
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        // Draw tiles
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (p, tile) in self.map.tiles.iter() {
            let r = vp.pos_to_rect(p);

            self.canvas
                .copy(
                    &self.atlas.texture[tile.index.z as usize],
                    Some(rect_to_sdl(tile.rect_padded(params.size))),
                    Some(rect_to_sdl(r)),
                )
                .unwrap();
        }

        // Draw debug stuff
        self.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (p, tile) in map.tiles.iter() {
            let mut r = vp.pos_to_rect(p);

            r.pos.x += 2;
            r.pos.y += 2;
            r.size.x -= 2 * 2;
            r.size.y -= 2 * 2;

            if let Tile::Doing = tile {
                self.canvas.draw_rect(rect_to_sdl(r)).unwrap();
            }
        }

        // Finish frame
        self.canvas.present();
    }

    pub fn output_size(&self) -> Vector2<u32> {
        let (x, y) = self.canvas.output_size().unwrap();
        Vector2::new(x, y)
    }
}
