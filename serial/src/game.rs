/*
   use serde::Deserialize;
   use serde::Serialize;
   use std::collections::HashMap;
   use std::io;
   use std::io::*;
   use std::result::Result::*;

   use crate::state::*;
   */

use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::*;
use sdl2::rect::*;
use std::time::Duration;

pub struct State {
    ctx:   sdl2::Sdl,
    event: sdl2::EventPump,
    canvas: sdl2::render::Canvas<sdl2::video::Window>,
    message: String,
    i: f32,
    x: i32,
    y: i32,
}


pub struct Tile {
    px: Vec<u8>
}

impl State {
    pub fn new() -> State {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("rust-sdl2 demo", 800, 600)
            .position_centered()
            .build()
            .unwrap();

        let event = sdl_context.event_pump().unwrap();
        let canvas = window.into_canvas().present_vsync().build().unwrap();

        State {
            ctx   : sdl_context,
            event : event,
            canvas: canvas,
            message: "Hello".to_string(),
            i: 0f32,
            x: 0,
            y: 0,
        }
    }

    pub fn update(&mut self) -> bool {
        let mut quit = false;

        println!("ok: {}", self.message);


        self.i += 1.0 / 60.0 / 4.0;

        if self.i > 1.0 {
            self.i = 0.0;
        }

        let r = ((self.i + 0.0/3.0) * 2.0 * std::f32::consts::PI).sin()*0.5+0.5;
        let g = ((self.i + 1.0/3.0) * 2.0 * std::f32::consts::PI).sin()*0.5+0.5;
        let b = ((self.i + 2.0/3.0) * 2.0 * std::f32::consts::PI).sin()*0.5+0.5;

        fn mk_color(i: f32) -> u8 { (i*255.0).floor() as u8 }

        // self.canvas.clear();
        for event in self.event.poll_iter() {
            match event {
                Event::Quit {..} => { quit = true; }
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => { quit = true; },
                Event::KeyDown { keycode: Some(Keycode::C), .. } => { self.canvas.clear(); },
                Event::MouseMotion {x, y, ..} => {
                    self.x = x as i32;
                    self.y = y as i32;
                }

                _ => {}
            }
        }
        // The rest of the game loop goes here...

        {
            let w = 42;
            self.canvas.set_draw_color(Color::RGB(0, 0, 0));
            self.canvas.fill_rect(Rect::new(self.x-w, self.y-w, 2*w as u32, 2*w as u32)).unwrap();
        }
        {
            let w = 40;
            self.canvas.set_draw_color(Color::RGB(mk_color(r), mk_color(g), mk_color(b)));
            self.canvas.fill_rect(Rect::new(self.x-w, self.y-w, 2*w as u32, 2*w as u32)).unwrap();
        }
        self.canvas.present();
        return quit;
    }
}
