use crate::atlas::Atlas;
use crate::input::SDLInput;
use crate::main2::Config;
use crate::main2::Tile;
use crate::main2::TileMap;
use crate::rect_to_sdl;
use fractal_toy::math::Rect as MRect;
use fractal_toy::math::*;
use fractal_toy::AtlasRegion;
use fractal_toy::AtlasTextureProvider;
use fractal_toy::Viewport;
use fractal_toy::TEXTURE_SIZE;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::Texture;
use sdl2::render::{BlendMode, Canvas};
use sdl2::video::Window;
use tilemap::CompareIter;
use tilemap::ComparedValue;

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
}

impl Sdl {
    pub fn new() -> Self {
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

            input: SDLInput::new(1.0 / 60.0),
            map: tilemap::TileMap::new(),
            atlas: Atlas::new(),
        }
    }

    pub fn events(&mut self) -> &mut SDLInput {
        let input = &mut self.input;
        let es = self.event.poll_iter().collect::<Vec<_>>();
        input.handle_sdl(&es);

        for e in es.iter() {
            match e {
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::Resized(x, y) => {
                        let window_size = Vector2::new((*x as u32).max(1), (*y as u32).max(1));
                        input.resize = Some(window_size);
                    },
                    _ => (),
                },
                _ => (),
            }
        }

        input
    }

    pub fn render(&mut self, map: &TileMap, vp: &Viewport) {
        self.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.canvas.clear();

        let sz = vp.size_in_pixels();
        self.canvas.set_draw_color(Color::RGB(255, 0, 255));
        self.canvas
            .draw_rect(Rect::new(10, 10, sz.x as u32 - 20, sz.y as u32 - 20))
            .unwrap();

        self.canvas.set_draw_color(Color::RGB(255, 0, 0));

        let iter = CompareIter::new(map.tiles.iter(), self.map.tiles.iter(), |(l, _), (r, _)| {
            l.cmp(&r)
        });
        let mut new_map = std::collections::BTreeMap::new();
        for i in iter {
            match i {
                ComparedValue::Left((pos, v)) => {
                    // only in map.tiles
                    if let Tile::Done(px) = v {
                        let texture_creator = self.canvas.texture_creator();
                        let region = self.atlas.alloc(&texture_creator, px);
                        new_map.insert(*pos, region);
                    }
                },

                ComparedValue::Right(_) => {
                    // dont insert
                },

                ComparedValue::Both((pos, _), (_, w)) => {
                    new_map.insert(*pos, w.clone());
                },
            }
        }

        self.map.tiles = new_map;

        for (p, tile) in self.map.tiles.iter() {
            let r = vp.pos_to_rect(p);

            self.canvas
                .copy(
                    &self.atlas.texture[tile.index.z as usize],
                    Some(rect_to_sdl(tile.rect_padded())),
                    Some(rect_to_sdl(r)),
                )
                .unwrap();
        }

        for (p, tile) in map.tiles.iter() {
            let mut r = vp.pos_to_rect(p);

            r.pos.x += 5;
            r.pos.y += 5;
            r.size.x -= 2 * 5;
            r.size.y -= 2 * 5;

            // // atlas.draw(sdl, tile, r);
            // self.canvas_copy(
            //     &self.atlas.texture[tile.index.z as usize],
            //     Some(rect_to_sdl(tile.rect_padded())),
            //     Some(rect_to_sdl(r)),
            // );

            if let Tile::Doing = tile {
                self.canvas.set_draw_color(Color::RGB(255, 255, 255));
                self.canvas.draw_rect(rect_to_sdl(r)).unwrap();
            }
        }

        self.canvas.present();
    }

    pub fn canvas_copy(
        &mut self,
        texture: &sdl2::render::Texture,
        src: Option<Rect>,
        dst: Option<Rect>,
    ) {
    }

    pub fn output_size(&self) -> Vector2<u32> {
        let (x, y) = self.canvas.output_size().unwrap();
        Vector2::new(x, y)
    }
}
