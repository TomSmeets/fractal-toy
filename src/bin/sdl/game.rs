use crate::atlas::Atlas;
use crate::sdl::Sdl;
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use serde::{Deserialize, Serialize};
use serial::atlas::AtlasRegion;
use serial::math::*;
use serial::tilemap::Task;
use serial::time::DeltaTime;
use serial::ui;
use serial::ui::UI;
use serial::{Fractal, Input};

#[derive(Serialize, Deserialize)]
pub struct State {
    #[serde(skip)]
    sdl: Sdl,

    #[serde(skip, default = "UI::new")]
    ui: UI,

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
        let ui = UI::new();

        let window_size = sdl.output_size();
        let fractal = Fractal::new(window_size);

        // TODO: get window size
        State {
            sdl,
            ui,
            input,
            fractal,
            window_size,
            atlas: Atlas::new(),
        }
    }

    pub fn update(&mut self) -> bool {
        let time = DeltaTime(1.0 / 60.0);

        // input stuff
        let events: Vec<_> = self.sdl.event.poll_iter().collect();
        self.input.begin();
        self.input.handle_sdl(&events);
        for e in events.iter() {
            match e {
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::Resized(x, y) => {
                        self.window_size = Vector2::new((*x as u32).max(1), (*y as u32).max(1));
                        self.fractal.pos.resize(self.window_size);
                    },
                    _ => (),
                },
                _ => (),
            }
        }

        let ui_input = ui::Input {
            viewport: V2i::new(self.window_size.x as i32, self.window_size.y as i32),
            mouse: self.input.mouse,
            left: self.input.mouse_down,
            right: false,
        };
        self.ui.input(ui_input);
        self.ui.update(&mut self.fractal);

        if !self.ui.has_focus() {
            // update fractal tiles
            self.input.execute(&mut self.fractal, time);
        }

        if !self.input.pause {
            self.fractal
                .update_tiles(&mut self.atlas.provider(&mut self.sdl));
        }

        // start drawing
        self.sdl.canvas.set_draw_color(Color::RGB(0, 0, 0));
        self.sdl.canvas.clear();

        // draw tiles
        self.sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
        for (p, tile) in self.fractal.tiles.iter() {
            let r = self.fractal.pos.pos_to_rect(p);
            match tile {
                Task::Done(tile) => {
                    // atlas.draw(sdl, tile, r);
                    self.sdl.canvas_copy(
                        &self.atlas.texture[tile.index.z as usize],
                        Some(tile.rect_padded().to_sdl()),
                        Some(r.to_sdl()),
                    );

                    if self.input.debug {
                        self.sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
                        self.sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },

                Task::Todo => {
                    if self.input.debug {
                        self.sdl.canvas.set_draw_color(Color::RGB(0, 0, 255));
                        self.sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },
                Task::Doing => {
                    if self.input.debug {
                        self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
                        self.sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },

                Task::Empty(_) => {
                    if self.input.debug {
                        self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 255));
                        self.sdl.canvas.draw_rect(r.to_sdl()).unwrap();
                    }
                },
            }
        }

        // draw debug
        if self.input.debug {
            // Show atlas
            // TODO: show in ui window?
            let w = self.window_size.x as i32 / self.atlas.texture.len().max(4) as i32;
            for (i, t) in self.atlas.texture.iter().enumerate() {
                self.sdl
                    .canvas_copy(t, None, Some(Rect::new(i as i32 * w, 0, w, w).to_sdl()));
            }
        }

        draw_ui(&self.ui, &mut self.sdl);

        self.sdl.canvas.present();

        if self.input.quit {
            self.input.quit = false;
            true
        } else {
            false
        }
    }
}

fn draw_ui(ui: &UI, sdl: &mut Sdl) {
    for (rect, rgb) in ui.rects.iter() {
        sdl.canvas
            .set_draw_color(Color::RGB(rgb[0], rgb[1], rgb[2]));
        sdl.canvas.fill_rect(rect.to_sdl()).unwrap();
    }
}
