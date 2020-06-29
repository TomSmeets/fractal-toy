use crate::atlas::Atlas;

use crate::input::SDLInput;
use crate::rect_to_sdl;
use crate::sdl::Sdl;
use fractal_toy::atlas::AtlasRegion;
use fractal_toy::fractal::FractalSave;
use fractal_toy::math::*;
use fractal_toy::state::Reload;
use fractal_toy::time::DeltaTime;
use fractal_toy::ui;
use fractal_toy::ui::UI;
use fractal_toy::{Fractal, Input};
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use sdl2::render::Texture;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tilemap::Task;

pub struct State {
    sdl: Sdl,
    ui: UI,
    uitxt: Option<UITextures>,

    pub input: SDLInput,

    fractal: Fractal<AtlasRegion>,

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
        let input = SDLInput {
            input: Input::new(),
        };
        let ui = UI::new();

        let window_size = sdl.output_size();
        let fractal = Fractal::new(window_size);

        State {
            sdl,
            ui,
            uitxt: None,
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
        self.input.input.begin();
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
            mouse: self.input.input.mouse,
            left: self.input.input.mouse_down,
            right: false,
        };
        self.ui.input(ui_input);
        self.ui.update(&mut self.fractal);

        if !self.ui.has_focus() {
            // update fractal tiles
            self.input.input.execute(&mut self.fractal, time);
        }

        if !self.input.input.pause {
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

            // atlas.draw(sdl, tile, r);
            self.sdl.canvas_copy(
                &self.atlas.texture[tile.index.z as usize],
                Some(rect_to_sdl(tile.rect_padded())),
                Some(rect_to_sdl(r)),
            );

            if self.input.input.debug {
                self.sdl.canvas.set_draw_color(Color::RGB(255, 255, 255));
                self.sdl.canvas.draw_rect(rect_to_sdl(r)).unwrap();
            }
        }

        if self.input.input.debug {
            let ts = self.fractal.queue.tiles.lock_high();

            for (k, v) in ts.map.iter() {
                let r = self.fractal.pos.pos_to_rect(k);
                match v {
                    Task::Todo => {
                        self.sdl.canvas.set_draw_color(Color::RGB(0, 0, 255));
                        self.sdl.canvas.draw_rect(rect_to_sdl(r)).unwrap();
                    },
                    Task::Doing => {
                        self.sdl.canvas.set_draw_color(Color::RGB(255, 0, 0));
                        self.sdl.canvas.draw_rect(rect_to_sdl(r)).unwrap();
                    },
                    _ => (),
                }
            }
        }

        // draw debug
        if self.input.input.debug {
            // Show atlas
            // TODO: show in ui window?
            let w = self.window_size.x as i32 / self.atlas.texture.len().max(4) as i32;
            for (i, t) in self.atlas.texture.iter().enumerate() {
                self.sdl
                    .canvas_copy(t, None, Some(rect_to_sdl(Rect::new(i as i32 * w, 0, w, w))));
            }
        }

        draw_ui(&mut self.uitxt, &self.ui, &mut self.sdl);

        self.sdl.canvas.present();

        if self.input.input.quit {
            self.input.input.quit = false;
            true
        } else {
            false
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct StateSave {
    fractal: FractalSave,
    input: SDLInput,
}

impl Reload for State {
    type Storage = StateSave;

    fn load(&mut self, data: StateSave) {
        self.input = data.input;
        self.fractal.load(data.fractal);
    }

    fn save(&self) -> StateSave {
        StateSave {
            input: self.input.clone(),
            fractal: self.fractal.save(),
        }
    }
}

pub struct UIImage {
    txt: Texture,
}

impl UIImage {
    pub fn from_path(p: &[u8], sdl: &mut Sdl) -> Self {
        use png::BitDepth;
        use png::ColorType;
        use png::Decoder;
        use sdl2::render::BlendMode;

        let decoder = Decoder::new(p);
        let (info, mut reader) = decoder.read_info().unwrap();
        // Allocate the output buffer.
        let mut buf = vec![0; info.buffer_size()];
        reader.next_frame(&mut buf).unwrap();

        let mut txt = sdl.create_texture_static_rgba8(info.width, info.height);

        assert_eq!(info.bit_depth, BitDepth::Eight);
        assert_eq!(info.color_type, ColorType::RGBA);
        txt.update(None, &buf, info.line_size).unwrap();

        txt.set_blend_mode(BlendMode::Blend);
        UIImage { txt }
    }
}

struct UITextures {
    map: HashMap<String, UIImage>,
}

impl UITextures {
    pub fn get_missing(&self) -> &UIImage {
        self.map.get("missing").unwrap()
    }

    pub fn get(&self, name: &str) -> &UIImage {
        match self.map.get(name) {
            Some(t) => t,
            None => {
                println!("not found: {}", name);
                self.get_missing()
            },
        }
    }
}

#[rustfmt::skip]
fn draw_ui(txt: &mut Option<UITextures>, ui: &UI, sdl: &mut Sdl) {
    if txt.is_none() {
        let mut t = UITextures {
            map: HashMap::new(),
        };

        // TODO: improve better :)
        t.map.insert(String::from("missing"),           UIImage::from_path(include_bytes!("../../res/missing.png"), sdl));
        t.map.insert(String::from("button_back"),       UIImage::from_path(include_bytes!("../../res/button_back.png"), sdl));
        t.map.insert(String::from("button_front_norm"), UIImage::from_path(include_bytes!("../../res/button_front_norm.png"), sdl));
        t.map.insert(String::from("button_front_down"), UIImage::from_path(include_bytes!("../../res/button_front_down.png"), sdl));
        t.map.insert(String::from("button_front_hot"),  UIImage::from_path(include_bytes!("../../res/button_front_hot.png"), sdl));
        t.map.insert(String::from("slider"),            UIImage::from_path(include_bytes!("../../res/slider.png"), sdl));
        t.map.insert(String::from("fractal_mandel"),    UIImage::from_path(include_bytes!("../../res/fractal_mandel.png"), sdl));
        t.map.insert(String::from("fractal_ship"),      UIImage::from_path(include_bytes!("../../res/fractal_ship.png"), sdl));
        t.map.insert(String::from("fractal_hybrid"),    UIImage::from_path(include_bytes!("../../res/fractal_hybrid.png"), sdl));
        t.map.insert(String::from("fractal_missing"),   UIImage::from_path(include_bytes!("../../res/fractal_missing.png"), sdl));

        *txt = Some(t);
    }

    let txt = txt.as_mut().unwrap();
    for (rect, name) in ui.rects.iter() {
        let img = txt.get(name);
        sdl.canvas_copy(&img.txt, None, Some(rect_to_sdl(*rect)));
    }
}
