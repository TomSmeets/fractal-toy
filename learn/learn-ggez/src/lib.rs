use ggez::event::EventHandler;
use ggez::graphics;
use ggez::{Context, GameResult};
use std::result::Result::*;

use ggez::conf::NumSamples;
use ggez::graphics::*;

use nalgebra as na;

type P2 = na::Point2<f32>;
type V2 = na::Vector2<f32>;

pub struct MyGame {
  canvas: Option<Canvas>,
  screen: graphics::Rect,
}

impl MyGame {
    pub fn new(ctx: &mut Context) -> GameResult<MyGame> {
        Ok(MyGame {
            screen: Rect::new(0.0, 0.0, 800.0, 600.0),
            canvas: None,
        })
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        let size = 512;
        let mut image_rgba : Vec<u8> = vec![255; size * size * 4];
        for y in 0..size {
            for x in 0..size {
                if x <= 4 || y <= 4 || x >= size - 5 || y >= size - 5 {
                    image_rgba[(y * size + x) * 4 + 0] = 0;
                    image_rgba[(y * size + x) * 4 + 1] = 255;
                    image_rgba[(y * size + x) * 4 + 2] = 0;
                    image_rgba[(y * size + x) * 4 + 3] = 255;
                } else {
                    image_rgba[(y * size + x) * 4 + 0] = (x * 255 / size) as u8;
                    image_rgba[(y * size + x) * 4 + 1] = 0;
                    image_rgba[(y * size + x) * 4 + 2] = (y * 255 / size) as u8;
                    image_rgba[(y * size + x) * 4 + 3] = 255;
                }
            }
        }

        let size = size as u16;
        let img = Image::from_rgba8(ctx, size, size, &image_rgba)?;
        let canvas: Canvas = Canvas::new(ctx, size, size, NumSamples::One)?;

        graphics::set_canvas(ctx, Some(&canvas));
        graphics::set_screen_coordinates(ctx, graphics::Rect::new(0.0, 0.0, size as f32, size as f32))?;
        graphics::clear(ctx, Color::from_rgba(255, 0, 255, 255));
        graphics::draw(ctx, &img, DrawParam::default())?;
        graphics::set_canvas(ctx, None);
        graphics::set_screen_coordinates(ctx, self.screen)?;
        self.canvas = Some(canvas);
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);
        if let Some(canvas) = &self.canvas {
            graphics::draw(ctx, canvas.image(), DrawParam::default())?;
        }
        graphics::present(ctx)
    }

    // update canvas size on window reisze
    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        let new_rect = graphics::Rect::new(0.0, 0.0, width as f32, height as f32);
        self.screen = new_rect;
    }
}
