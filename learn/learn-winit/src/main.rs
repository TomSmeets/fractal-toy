use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

mod platform {}

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

        let canvas = window.canvas();
        // js! {
        //     @{&canvas}.removeAttribute("width");
        //     @{&canvas}.removeAttribute("height");
        //     @{&canvas}.style = "";
        // }
    }

    let mut gilrs = gilrs::Gilrs::new().unwrap();

    stdweb::console!(log, "%s", format!("{:#?}", gilrs));

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        // println!("{:?}", event);

        // stdweb::console!(log, "%s", format!("{:?}", event));

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                window_id,
            } => (),
            // stdweb::console!(log, "%s", format!("size: {:?}", size)),
            Event::MainEventsCleared => {
                while let Some(ev) = gilrs.next_event() {
                    stdweb::console!(log, "%s", format!("{:?}", ev));
                }

                window.request_redraw();
            }
            _ => (),
        }
    });
}
