use crate::atlas::Atlas;
use crate::sdl::Sdl;
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use serde::{Deserialize, Serialize};
use serial::atlas::AtlasRegion;
use serial::math::*;
use serial::tilemap::Task;
use serial::time::DeltaTime;
use serial::{Fractal, Input};

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    sdl: Sdl,

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
        input.execute(fractal, time);

        if !input.pause {
            fractal.update_tiles(&mut atlas.provider(sdl));
        }

        // start drawing
        sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        sdl.canvas.clear();

        // draw tiles
        sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (p, tile) in fractal.tiles.iter() {
            let r = fractal.pos.pos_to_rect(p);
            match tile {
                Task::Done(tile) => {
                    // atlas.draw(sdl, tile, r);
                    sdl.canvas_copy(
                        &atlas.texture[tile.index.z as usize],
                        Some(tile.rect_padded().to_sdl()),
                        Some(r.to_sdl()),
                    );

                    if input.debug {
                        sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
                        sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },

                Task::Todo => {
                    if input.debug {
                        sdl.canvas.set_draw_color(Color::RGB(0, 0, 255));
                        sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },
                Task::Doing => {
                    if input.debug {
                        sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
                        sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },

                Task::Empty(_) => {
                    if input.debug {
                        sdl.canvas.set_draw_color(Color::RGB(255, 0, 255));
                        sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },
            }
        }

        // draw debug
        if input.debug {
            // Show atlas
            // TODO: show in ui window?
            let w = window_size.x as i32 / atlas.texture.len().max(4) as i32;
            for (i, t) in atlas.texture.iter().enumerate() {
                sdl.canvas_copy(t, None, Some(Rect::new(i as i32 * w, 0, w, w).to_sdl()));
            }
        }

        // draw slider
        {
            let w = 45;
            let pad = 10;
            let rect = Rect::new(
                window_size.x as i32 - w - pad,
                pad,
                w,
                window_size.y as i32 - pad * 2,
            );
            let slider_x = window_size.x as i32 - w / 2 - pad;
            {
                let rect = Rect::new(slider_x - 10, rect.pos.y, 20, rect.size.y);
                sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
                sdl.canvas.fill_rect(rect.to_sdl()).unwrap();
                sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
                sdl.canvas.draw_rect(rect.to_sdl()).unwrap();
            }

            {
                let z = (fractal.pos.zoom + 2.5) / (2.5 + 48.5);
                let z = z.max(0.0).min(1.0);
                let h = (z * rect.size.y as f64) as i32;
                let slider_radius = 10;
                let r_slider = Rect::new(
                    rect.pos.x,
                    rect.pos.y + h - slider_radius,
                    rect.size.x,
                    slider_radius * 2,
                );
                sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
                sdl.canvas.fill_rect(r_slider.to_sdl()).unwrap();
            }
        }
        {
            let mut x = 0;
            let w = 45;
            let pad = 10;

            sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
            for _ in 0..6 {
                let rect = Rect::new(x + pad, window_size.y as i32 - w - pad, w, w);
                sdl.canvas.fill_rect(rect.to_sdl()).unwrap();
                x += w + pad;
            }
        }

        sdl.canvas.present();

        if input.quit {
            input.quit = false;
            true
        } else {
            false
        }
    }
}
