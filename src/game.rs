use crate::module::{input::InputAction, Fractal, Input, Sdl};
use serde::{Deserialize, Serialize};

use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;

use crate::math::*;
use crate::module::atlas::Atlas;
use crate::module::atlas::AtlasRegion;
use crate::module::atlas::AtlasTextureCreator;
use crate::module::time::DeltaTime;

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    sdl: Sdl,
    #[serde(skip)]
    pub input: Input,

    fractal: Fractal<AtlasRegion>,

    #[serde(skip)]
    atlas: Atlas,
    window_size: Vector2<u32>,
}

impl Default for State {
    fn default() -> State {
        State::new()
    }
}

impl State {
    pub fn unload(&mut self) {}

    pub fn reload(&mut self) {}

    pub fn new() -> State {
        let sdl = Sdl::new();
        let input = Input::new();

        let window_size = sdl.output_size();
        let fractal = Fractal::new(window_size);

        // TODO: get window size
        State {
            sdl,
            input,
            fractal,
            window_size,
            atlas: Atlas::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let State {
            sdl,
            input,
            window_size,
            fractal,
            atlas,
        } = self;
        let time = DeltaTime(1.0 / 60.0);

        // input stuff
        let events: Vec<_> = sdl.event.poll_iter().collect();
        input.begin();
        input.handle_sdl(&events);
        for e in events.iter() {
            match e {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(x, y) => {
                        *window_size = Vector2::new((*x as u32).max(1), (*y as u32).max(1));
                        fractal.pos.resize(*window_size);
                    },
                    _ => (),
                },
                _ => (),
            }
        }

        // update fractal tiles
        fractal.do_input(input, time);

        if !fractal.pause {
            fractal.update_tiles(&mut AtlasTextureCreator { sdl, atlas });
        }

        // start drawing
        sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl.canvas.clear();

        // draw tiles
        sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (p, tile) in fractal.tiles.iter() {
            let r = fractal.pos.pos_to_rect(&p.pos);
            atlas.draw(sdl, tile, r);
            if fractal.debug {
                sdl.canvas.draw_rect(r.to_sdl()).unwrap();
            }
        }

        // draw debug
        if fractal.debug {
            // visualize queue
            {
                let q = fractal.queue.lock().unwrap();
                sdl.canvas.set_draw_color(Color::RGB(0, 0, 255));
                for p in q.todo.iter() {
                    let r = fractal.pos.pos_to_rect(&p.pos);
                    sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                }
                sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
                for p in q.doing.iter() {
                    let r = fractal.pos.pos_to_rect(&p.pos);
                    sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                }
            }

            // Show atlas
            // TODO: show in ui window?
            let w = window_size.x as i32 / atlas.texture.len().max(4) as i32;
            for (i, t) in atlas.texture.iter().enumerate() {
                sdl.canvas_copy(t, None, Some(Rect::new(i as i32 * w, 0, w, w).to_sdl()));
            }
        }

        sdl.canvas.present();

        input.button(InputAction::Quit).is_down
    }
}
