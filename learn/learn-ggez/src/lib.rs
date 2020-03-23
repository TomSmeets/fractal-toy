use ggez::event::EventHandler;
use ggez::graphics;
use ggez::{Context, GameResult};
use std::result::Result::*;

use ggez::graphics::*;
use nalgebra as na;

type P2 = na::Point2<f32>;
type V2 = na::Vector2<f32>;

pub struct MyGame {}

impl MyGame {
    pub fn new(ctx: &mut Context) -> MyGame {
        MyGame {}
    }
}

impl EventHandler for MyGame {
    fn update(&mut self, _ctx: &mut Context) -> GameResult<()> {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        graphics::clear(ctx, graphics::WHITE);

        graphics::draw(
            ctx,
            &Text::new("Hello world"),
            DrawParam::default().color(Color::from_rgb(0, 0, 0)),
        )?;

        {
            // Look at imgui-ggez-starter for texture creation. we need to create a gfx texture to be abel to update it
            let pos = ggez::input::mouse::position(ctx);
            let t = Text::new("How are you");
            let p = DrawParam::default()
                .color(Color::from_rgb(0, 0, 0))
                .dest(pos);
            graphics::draw(ctx, &t, p)?;
        }
        graphics::present(ctx)
    }
}
