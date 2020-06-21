use crate::atlas::Atlas;
use crate::sdl::Sdl;
use sdl2::event::{Event, WindowEvent};
use sdl2::pixels::Color;
use sdl2::render::Texture;
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

    #[serde(skip)]
    uitxt: Option<UITextures>,

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

        draw_ui(&mut self.uitxt, &self.ui, &mut self.sdl);

        self.sdl.canvas.present();

        if self.input.quit {
            self.input.quit = false;
            true
        } else {
            false
        }
    }
}

pub struct UIImage {
    txt: Texture,
}

use std::path::Path;
impl UIImage {
    pub fn from_path(p: &Path, sdl: &mut Sdl) -> Self {
        use png::BitDepth;
        use png::ColorType;
        use png::Decoder;
        use sdl2::render::BlendMode;
        use std::fs::File;

        let decoder = Decoder::new(File::open(p).unwrap());
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

use std::collections::HashMap;

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

fn draw_ui(txt: &mut Option<UITextures>, ui: &UI, sdl: &mut Sdl) {
    if txt.is_none() {
        let mut t = UITextures { map: HashMap::new() };
        t.map.insert(String::from("missing"),           UIImage::from_path(Path::new("./res/color.png"),               sdl));
        t.map.insert(String::from("button_back"),       UIImage::from_path(Path::new("./res/button_back.png"),         sdl));
        t.map.insert(String::from("button_front_norm"), UIImage::from_path(Path::new("./res/button_front_norm.png"),   sdl));
        t.map.insert(String::from("button_front_down"), UIImage::from_path(Path::new("./res/button_front_down.png"),   sdl));
        t.map.insert(String::from("button_front_hot"),  UIImage::from_path(Path::new("./res/button_front_hot.png"),    sdl));
        t.map.insert(String::from("slider"),            UIImage::from_path(Path::new("./res/slider.png"),              sdl));
        t.map.insert(String::from("fractal_mandel"),    UIImage::from_path(Path::new("./res/fractal_mandel.png"),      sdl));
        t.map.insert(String::from("fractal_ship"),      UIImage::from_path(Path::new("./res/fractal_ship.png"),        sdl));
        t.map.insert(String::from("fractal_hybrid"),    UIImage::from_path(Path::new("./res/fractal_hybrid.png"),      sdl));
        t.map.insert(String::from("fractal_missing"),    UIImage::from_path(Path::new("./res/fractal_missing.png"),      sdl));
        *txt = Some(t);
    }

    let txt = txt.as_mut().unwrap();
    for (rect, name) in ui.rects.iter() {
        let img = txt.get(name);
        sdl.canvas_copy(&img.txt, None, Some(rect.to_sdl()));
    }
}
