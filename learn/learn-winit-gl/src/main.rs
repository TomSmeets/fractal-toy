use glutin::event::Event;
use glutin::event_loop::ControlFlow;

pub mod gfx;
mod gl;
mod platform;
mod state;

pub use platform::Platform;
pub use state::State;

pub static mut PLATFORM: Option<Platform> = None;

pub fn platform() -> &'static mut Platform {
    unsafe { PLATFORM.as_mut().unwrap() }
}

fn main() {
    println!("Hello, world!");
    let (platform, event_loop) = Platform::new();

    unsafe {
        PLATFORM = Some(platform);
    };
    let platform = unsafe { PLATFORM.as_mut().unwrap() };

    let mut state = State::new(platform);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        state.event(&event);
        if let Event::MainEventsCleared { .. } = event {
            state.update(platform);

            if state.quit {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
