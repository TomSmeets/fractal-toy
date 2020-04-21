use image::png::PNGEncoder;
use image::ColorType;
use serial::module::fractal::TileTextureProvider;
use serial::module::fractal::TEXTURE_SIZE;
use serial::module::Fractal;
use stdweb::js;
use stdweb::unstable::TryInto;
use stdweb::web::html_element::ImageElement;
use stdweb::web::*;

use serial::math::*;

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

        {
            let mut png = PNGEncoder::new(&mut bytes);
            png.encode(
                px,
                TEXTURE_SIZE as u32,
                TEXTURE_SIZE as u32,
                ColorType::Rgba8,
            );
        }
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
    fractal: Fractal<ImageElement>,
}

impl State {
    fn new() -> Self {
        State {
            time: 0.0,
            fractal: Fractal::new(Vector2::new(600, 800)),
        }
    }

    fn update(&mut self, time: f64) {
        let dt = time - self.time;
        self.time = time;

        let mut prov = Provider {};
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

    let state = Box::new(State::new());
    animate(state, 0.0);
}
