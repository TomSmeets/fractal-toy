#[macro_use]
extern crate stdweb;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use std::time::Instant;

mod platform {}

#[cfg(feature = "web")]
macro_rules! println {
    ($($token:tt)*) => {
        let string = format!( $($token)+ );
        stdweb::js! {
            console.log( @{string} );
        }
    };
}

pub fn main() {
    let mut window = WindowBuilder::new()
        .with_title("A fantastic window!")
        .with_resizable(true);

    #[cfg(feature = "web")]
    {
        use stdweb::unstable::TryInto;
        use stdweb::web::document;
        use stdweb::web::html_element::CanvasElement;
        use stdweb::web::INonElementParentNode;
        use winit::platform::web::WindowBuilderExtStdweb;
        let canvas: CanvasElement = document()
            .get_element_by_id("canvas")
            .expect("Get HTML body")
            .try_into()
            .unwrap();
        window = window.with_canvas(Some(canvas));
    }

    let event_loop = EventLoop::new();
    let window = window.build(&event_loop).unwrap();
    #[cfg(feature = "web")]
    {
        // ignore window size set by winit, the size is set with css in index.html
        use stdweb::js;
        use stdweb::web::event::ResizeEvent;
        use stdweb::web::html_element::CanvasElement;
        use stdweb::web::INonElementParentNode;
        use stdweb::web::{document, IEventTarget, IHtmlElement, IParentNode, TypedArray};
        use winit::platform::web::WindowExtStdweb;
        let canvas = window.canvas();
        stdweb::web::window().add_event_listener(move |_: ResizeEvent| {
            canvas.set_width(canvas.offset_width() as u32);
            canvas.set_height(canvas.offset_height() as u32);
        });
    }

    let mut gilrs = gilrs::Gilrs::new().unwrap();

    println!("gilrs: {:#?}", gilrs);

    let mut time_old = Instant::now();

    event_loop.run(move |event, runner, control_flow| {
        *control_flow = ControlFlow::Wait;

        // println!("{:?}", event);

        // stdweb::console!(log, "%s", format!("{:?}", event));

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => (),
            // stdweb::console!(log, "%s", format!("size: {:?}", size)),
            Event::MainEventsCleared => {
                while let Some(ev) = gilrs.next_event() {
                    println!("event {:?}", ev);
                }

                let time = Instant::now();
                let dt = (time - time_old).as_secs();
                println!("dt: {}", dt);
                time_old = time;

                // window.request_redraw();
            }
            _ => (),
        }
    });
}
