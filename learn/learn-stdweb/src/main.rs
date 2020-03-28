use stdweb::js;
use stdweb::unstable::TryInto;
use stdweb::web::event::ResizeEvent;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::*;
use stdweb::traits::*;

mod webgl_rendering_context;
use webgl_rendering_context::{WebGLBuffer, WebGLRenderingContext as Gl, WebGLUniformLocation};


struct Logger {
    body: Element,
}

impl Logger {
    fn push(&self, key: &str, mut value: String) {
        self.body.set_text_content(&value);
    }

    fn clear(&self) {
        for n in self.body.child_nodes().iter() {
           self.body.remove_child(&n);
        }
    }
}
struct State {
    time: f64,
    time_left: f64,
    count: i32,
    logger: Logger,
    canvas: CanvasElement,
    context: Gl,
}

impl State {
    fn new() {
    }

    fn step(&mut self) {
        self.logger.push("state", format!("count: {}", self.count));
        self.count+=1;
    }

    fn update(&mut self, time: f64) {
        let time = time / 1000.0;
        let dt   = time - self.time;
        self.time = time;
        self.time_left += dt;

        self.context.clear_color(1.0, 0.0, 1.0, 1.0);
        self.context.clear(Gl::COLOR_BUFFER_BIT);

        while self.time_left > 1.0 {
            self.step();
            self.time_left -= 1.0;
        }
    }

    fn animate(mut self: Box<Self>, time: f64) {
        self.update(time);
        window().request_animation_frame(move |time| {
            self.animate(time);
        });
    }
}

fn main() {
    stdweb::initialize();
    let log: Element = document().query_selector("#log-info").unwrap().unwrap();
    let canvas: CanvasElement = document()
        .query_selector("#canvas")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    let context: Gl = canvas.get_context().unwrap();

    let mut logger = Logger { body : log };
    logger.push("info", "Hello world".to_string());

/*
    canvas.add_event_listener({
        move |_: ResizeEvent| {
            let w = canvas.offset_width();
            let h = canvas.offset_height();
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
            logger.push("resize", format!("{}x{}", w, h));
        }
    });
    */

    let state = Box::new(State {
        time: 0.0,
        time_left: 0.0,
        count: 0,
        logger,
        canvas,
        context,
    });


    state.animate(0.0);

    // let context: gl = canvas.get_context().unwrap();
    stdweb::event_loop();
}
