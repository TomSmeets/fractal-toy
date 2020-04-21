use image::png::PNGEncoder;
use image::ColorType;
use serial::fractal::TileTextureProvider;
use serial::fractal::TEXTURE_SIZE;
use serial::time::DeltaTime;
use serial::Fractal;
use serial::Input;
use stdweb::js;
use stdweb::unstable::TryInto;
use stdweb::web::event::ClickEvent;
use stdweb::web::html_element::ImageElement;
use stdweb::web::html_element::InputElement;
use stdweb::web::*;

use serial::math::*;
use std::cell::RefCell;
use std::rc::Rc;

struct Provider {}
impl TileTextureProvider for Provider {
    type Texture = ImageElement;

    fn alloc(&mut self, px: &[u8]) -> ImageElement {
        let img: ImageElement = document()
            .create_element("img")
            .unwrap()
            .try_into()
            .unwrap();
        // img.set_src("https://upload.wikimedia.org/wikipedia/commons/thumb/2/21/Mandel_zoom_00_mandelbrot_set.jpg/1920px-Mandel_zoom_00_mandelbrot_set.jpg");
        let mut bytes = Vec::new();

        PNGEncoder::new(&mut bytes)
            .encode(
                px,
                TEXTURE_SIZE as u32,
                TEXTURE_SIZE as u32,
                ColorType::Rgba8,
            )
            .unwrap();

        let bytes = base64::encode(bytes);
        let src = format!("data:image/png;base64,{}", &bytes);
        img.set_src(&src);
        document().body().unwrap().append_child(&img);
        img
    }

    fn free(&mut self, img: ImageElement) {
        img.remove();
    }
}

struct State {
    time: f64,
    input: Rc<RefCell<Input>>,
    fractal: Fractal<ImageElement>,
}

impl State {
    fn new(input: Rc<RefCell<Input>>) -> Self {
        State {
            time: 0.0,
            input,
            fractal: Fractal::new(Vector2::new(600, 800)),
        }
    }

    fn update(&mut self, time: f64) {
        let dt = DeltaTime((time - self.time) as f32);
        self.time = time;
        let mut prov = Provider {};
        {
            let mut input = self.input.borrow_mut();
            self.fractal.do_input(&input, dt);
            input.begin();
        }
        self.fractal.update_tiles(&mut prov);
        let msg = format!("tile_count: {}", self.fractal.tiles.tiles.len());
        js! {
            console.log(@{msg})
        }
    }
}

fn animate(mut state: Box<State>, time: f64) {
    state.update(time);
    window().request_animation_frame(move |time| {
        animate(state, time);
    });
}

fn main() {
    let msg = "Hello world!";
    js! {
        console.log(@{msg})
    }

    let input = Rc::new(RefCell::new(Input::new()));

    fn make_button<F: FnMut(&mut Input) + 'static>(
        input: &Rc<RefCell<Input>>,
        name: &str,
        mut fun: F,
    ) {
        let input = Rc::clone(input);
        let button: InputElement = document()
            .create_element("input")
            .unwrap()
            .try_into()
            .unwrap();
        button.set_attribute("type", "button").unwrap();
        button.set_attribute("value", name).unwrap();
        button.add_event_listener(move |_: ClickEvent| {
            let mut input = input.borrow_mut();
            fun(&mut input);
        });
        document().body().unwrap().append_child(&button);
    }

    make_button(&input, "cycle", |input| input.cycle = true);
    make_button(&input, "more iters", |input| input.iter_inc = true);
    make_button(&input, "less iters", |input| input.iter_dec = true);

    let state = Box::new(State::new(input));
    animate(state, 0.0);
}
