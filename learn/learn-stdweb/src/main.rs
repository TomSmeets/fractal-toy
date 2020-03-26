use stdweb::js;
use stdweb::unstable::TryInto;
use stdweb::web::event::ResizeEvent;
use stdweb::web::html_element::CanvasElement;
use stdweb::web::*;

mod webgl_rendering_context;
use webgl_rendering_context::{WebGLBuffer, WebGLRenderingContext as Gl, WebGLUniformLocation};

fn main() {
    stdweb::initialize();
    let log: Element = document().query_selector("#log").unwrap().unwrap();
    log.set_text_content("Hello world!");
    let canvas: CanvasElement = document()
        .query_selector("#canvas")
        .unwrap()
        .unwrap()
        .try_into()
        .unwrap();
    let context: Gl = canvas.get_context().unwrap();

    window().add_event_listener({
        move |_: ResizeEvent| {
            let w = canvas.offset_width();
            let h = canvas.offset_height();
            log.set_text_content(&format!("resize! size: {}x{}", w, h));
            canvas.set_width(w as u32);
            canvas.set_height(h as u32);
            context.clear_color(1.0, 0.0, 1.0, 1.0);
            context.clear(Gl::COLOR_BUFFER_BIT);
        }
    });

    // let context: gl = canvas.get_context().unwrap();
    stdweb::event_loop();
}
