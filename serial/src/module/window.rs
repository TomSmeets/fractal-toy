use crate::math::*;
use crate::module::Sdl;
use sdl2::event::{Event, WindowEvent};

pub struct Window {
    pub size: Vector2<u32>,
}

impl Window {
    pub fn new(sdl: &Sdl) -> Self {
        let (x, y) = sdl.canvas.output_size().unwrap();
        Window {
            size: Vector2::new(x, y),
        }
    }

    pub fn update(&mut self, sdl: &Sdl) {
        for event in &sdl.events {
            match event {
                Event::Window {
                    win_event: WindowEvent::Resized(x, y),
                    ..
                } => {
                    self.size.x = (*x as u32).max(1);
                    self.size.y = (*y as u32).max(1);
                },
                _ => {},
            }
        }
    }
}
