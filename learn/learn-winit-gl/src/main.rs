use glutin::event::Event;
use glutin::event_loop::ControlFlow;

mod gl;
mod platform;
mod state;

pub use platform::Platform;
pub use state::State;

fn main() {
    println!("Hello, world!");
    let (mut platform, event_loop) = Platform::new();
    let mut state = State::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        state.event(&event);
        if let Event::MainEventsCleared { .. } = event {
            state.update(&mut platform);

            if state.quit {
                *control_flow = ControlFlow::Exit;
            }
        }
    });
}
