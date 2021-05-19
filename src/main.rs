mod gpu;

use gpu::Gpu;
use winit::{event_loop::EventLoop, window::Window};

pub fn main() {
    let event_loop = EventLoop::new();
    let window = Window::new(&event_loop).unwrap();
    Gpu::init(&window);
    println!("hello world!");

    event_loop.run(|_, _, _| {});
}
