use ggez::conf::WindowMode;
use ggez::event;
use ggez::ContextBuilder;
use std::result::Result::*;

use learn_ggez::MyGame;

// Cons: around 4 second compile time overhead, however building in release is kind of the same time overhead
// Cons: huge dependency tree
// Con?: uses old gfx
// Pros: has a font by default
// Pros: supports gamepads (via gilrs)
fn main() {
    // Make a Context and an EventLoop.
    let (mut ctx, mut event_loop) = ContextBuilder::new("game_name", "author_name")
        .window_mode(WindowMode::default().resizable(true))
        .build()
        .unwrap();

    // Create an instance of your event handler.
    // Usually, you should provide it with the Context object
    // so it can load resources like images during setup.
    let mut my_game = MyGame::new(&mut ctx).unwrap();

    // Run!
    match event::run(&mut ctx, &mut event_loop, &mut my_game) {
        Ok(_) => println!("Exited cleanly."),
        Err(e) => println!("Error occured: {}", e),
    }
}
