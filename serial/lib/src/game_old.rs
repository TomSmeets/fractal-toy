use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::io;
use std::io::*;
use std::result::Result::*;

use crate::state::*;

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use std::time::Duration;

#[derive(Serialize, Deserialize)]
#[derive(Debug, PartialEq)]
#[derive(Default)]
#[serde(default)]
pub struct State {
    map: HashMap<String, String>,
    history: Vec<String>,
    number : i32,
}

impl State {
    pub fn new() -> State {
        {
            let sdl_context = sdl2::init().unwrap();
            let video_subsystem = sdl_context.video().unwrap();

            let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
                .position_centered()
                .build()
                .unwrap();

            let mut canvas = window.into_canvas().build().unwrap();

            canvas.set_draw_color(Color::RGB(0, 255, 255));
            canvas.clear();
            canvas.present();
            let mut event_pump = sdl_context.event_pump().unwrap();
            let mut i = 0;
            'running: loop {
                i = (i + 1) % 255;
                canvas.set_draw_color(Color::RGB(i, 64, 255 - i));
                canvas.clear();
                for event in event_pump.poll_iter() {
                    match event {
                        Event::Quit {..} |
                            Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                                break 'running
                            },
                        _ => {}
                    }
                }
                // The rest of the game loop goes here...

                canvas.present();
                ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 60));
            }
        }

        match load("auto") {
            Ok(s)  => s,
            Err(_) => Default::default()
        }
    }

    pub fn update(&mut self) -> bool {
        let mut quit = false;
        let mut line = String::new();
        print!("/ ");
        io::stdout().flush().unwrap();
        io::stdin().read_line(&mut line).unwrap();

        if line.is_empty() {
            println!("END");
            save("auto", self);
            return true;
        }

        let split : Vec<&str> = line.split_whitespace().collect();

        if split.len() == 0 {
            return false;
        }

        match split[0] {
            "fun" => { println!("HIHI"); self.number += 4; }

            "list" => {
                for f in list() {
                    println!("{}", f);
                }
            }

            "save" => {
                save(split[1], self);
            }

            "load" => {
                match load(split[1]) {
                    Ok(s2) => { *self = s2; }
                    Err(e) => { println!("Failed to load {}\n{:?}", split[1], e) }
                };
            }

            "history" => {
                for (i, h) in self.history.iter().enumerate() {
                    println!("{:3}: {}", i, h);
                }
            }

            "set" => {
                self.map.insert(split[1].to_string(), split[2].to_string());
            }

            "get" => {
                println!("{}", self.map[split[1]]);
            }

            "all" => {
                let mut items : Vec<_> = self.map.iter().collect();
                items.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
                for (k, v) in &items {
                    println!("{} = {:?}", k, v);
                }
            }

            "quit" => {
                save("auto", self);
                quit = true;
            }

            cmd => {
                println!("unkonwn command: {}", cmd);
            }
        }

        self.history.push(String::from(line.trim()));
        return quit;
    }
}
