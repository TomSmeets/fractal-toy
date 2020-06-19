#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ColorScheme {
    width: usize,
    height: usize,
    // rgb
    data: Box<[u8]>,
}

impl ColorScheme {
    pub fn new() -> Self {
        let width = 32;
        let height = 32;

        let mut data = Vec::with_capacity(width * height * 4);

        for pixel_y in 0..height {
            let v = 1.0 - pixel_y as f64 / (height - 1) as f64;
            for pixel_x in 0..width {
                let u = pixel_x as f64 / (width - 1) as f64;
                data.extend(&hsv2rgb(u, v, v));
                data.push(255);
            }
        }

        //         {
        //             let file = File::create("./color.png").unwrap();
        //             let ref mut w = BufWriter::new(file);
        //             let mut encoder = png::Encoder::new(w, width as u32, height as u32); // Width is 2 pixels and height is 1.
        //             encoder.set_color(png::ColorType::RGBA);
        //             encoder.set_depth(png::BitDepth::Eight);
        //             let mut writer = encoder.write_header().unwrap();
        //             writer.write_image_data(&data).unwrap(); // Save
        //         }
        //
        //         let decoder = png::Decoder::new(File::open("./color-in.png").unwrap());
        //         let (info, mut reader) = decoder.read_info().unwrap();
        //         // Allocate the output buffer.
        //         let mut buf = vec![0; info.buffer_size()];
        //         // Read the next frame. Currently this function should only called once.
        //         // The default options
        //         reader.next_frame(&mut buf).unwrap();
        //
        //         let width = info.width as usize;
        //         let height = info.height as usize;
        //         let data = buf;
        //
        let data = data.into_boxed_slice();
        ColorScheme {
            width,
            height,
            data,
        }
    }

    pub fn get_px(&self, x: usize, y: usize) -> [u8; 3] {
        let x = x % self.width;
        let y = y % self.height;
        let i = y * self.width + x;
        let s = &self.data[i * 4..(i + 1) * 4];
        [s[2], s[1], s[0]]
    }

    pub fn get(&self, u: f64, v: f64) -> [u8; 3] {
        // let u = (iter / 64.0).fract();
        // let v = iter / max;

        let v = 1.0 - v;
        let x = u * (self.width - 1) as f64;
        let y = v * (self.height - 1) as f64;

        let lx = x.floor();
        let ly = y.floor();
        let hx = lx + 1.0;
        let hy = ly + 1.0;

        let pll = self.get_px(lx as usize, ly as usize);
        let plh = self.get_px(lx as usize, hy as usize);
        let phl = self.get_px(hx as usize, ly as usize);
        let phh = self.get_px(hx as usize, hy as usize);

        let fxl_0 = (hx - x) * pll[0] as f64 + (x - lx) * phl[0] as f64;
        let fxl_1 = (hx - x) * pll[1] as f64 + (x - lx) * phl[1] as f64;
        let fxl_2 = (hx - x) * pll[2] as f64 + (x - lx) * phl[2] as f64;

        let fxh_0 = (hx - x) * plh[0] as f64 + (x - lx) * phh[0] as f64;
        let fxh_1 = (hx - x) * plh[1] as f64 + (x - lx) * phh[1] as f64;
        let fxh_2 = (hx - x) * plh[2] as f64 + (x - lx) * phh[2] as f64;

        let fxy_0 = (hy - y) * fxl_0 + (y - ly) * fxh_0 as f64;
        let fxy_1 = (hy - y) * fxl_1 + (y - ly) * fxh_1 as f64;
        let fxy_2 = (hy - y) * fxl_2 + (y - ly) * fxh_2 as f64;

        [fxy_0 as u8, fxy_1 as u8, fxy_2 as u8]
    }
}

fn hsv2rgb(hue: f64, sat: f64, val: f64) -> [u8; 3] {
    let hue = hue.fract();
    let hue = hue * 6.0;
    let part = hue as u32;
    let fract = hue - part as f64;

    // upper limit
    let max = 255.0 * val;
    // lower limit
    let min = 255.0 * val - 255.0 * val * sat;
    // increasing slope
    let inc = fract * max + (1.0 - fract) * min;
    // decreasing slope
    let dec = fract * min + (1.0 - fract) * max;

    // as u8
    let min = min as u8;
    let max = max as u8;
    let inc = inc as u8;
    let dec = dec as u8;
    match part {
        0 => [max, inc, min],
        1 => [dec, max, min],
        2 => [min, max, inc],
        3 => [min, dec, max],
        4 => [inc, min, max],
        5 => [max, min, dec],
        _ => [max, max, max],
    }
}
