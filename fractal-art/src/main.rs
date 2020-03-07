use std::fs::File;
use std::io::BufWriter;
use std::mem;
use std::path::Path;

use rand::prelude::*;

// TODO: use image crate instad of png, and save as bmp

#[derive(Clone)]
#[repr(packed)]
struct Color {
    pub data: [u8; 3],
}

impl Color {
    fn mutate(&self, l: i32) -> Self {
        let mut r = self.data[0] as i32;
        let mut g = self.data[1] as i32;
        let mut b = self.data[2] as i32;

		let l = 2.5 as f32;
        r += (rand::random::<f32>() * 2.0 * l - l) as i32;
        g += (rand::random::<f32>() * 2.0 * l - l) as i32;
        b += (rand::random::<f32>() * 2.0 * l - l) as i32;

        if r > 255 { r = 255; }
        if r < 0   { r = 0;   }
        if g > 255 { g = 255; }
        if g < 0   { g = 0;   }
        if b > 255 { b = 255; }
        if b < 0   { b = 0;   }
        Color { data : [r as u8, g as u8, b as u8] }
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<u8>,
}


fn around(i: i32, j: i32, r: i32) -> Vec<(i32, i32)> {
    let mut xs = Vec::new();

    for o in -r..=r {
        xs.push((i + o, j + r));
        xs.push((i + o, j - r));
        xs.push((i + r, j + o));
        xs.push((i - r, j + o));
    }

    // TODO: Shuffle

    let mut rng = rand::thread_rng();
    xs.shuffle(&mut rng);
    xs
}

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            data: vec![0; (width * height * 3) as usize],
        }
    }

    fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Color> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            let cs = (&mut self.data[i * 3..(i + 1) * 3]).as_ptr() as *mut [u8; 3];
            unsafe { Some(mem::transmute(cs)) }
        }
    }

    fn at(&self, x: i32, y: i32) -> Option<Color> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            let cs = (&self.data[i * 3..(i + 1) * 3]).as_ptr() as *mut [u8; 3];
            unsafe {
                let c: &Color = mem::transmute(cs);
                Some(c.clone())
            }
        }
    }

    fn generate(&mut self) {
        // center
        let cx = (self.width / 2) as i32;
        let cy = (self.height / 2) as i32;
        let ring_count = (self.width / 2) as i32;

        {
            let p = self.at_mut(cx, cy).unwrap();
            p.data[0] = 255;
            p.data[1] = 0;
            p.data[2] = 0;
        }


        for r in 1..ring_count {
            println!("i = {}", r);
            let vs = around(cx, cy, r);
            for (x, y) in vs {
                let mut c: Option<Color> = None;
                let vs = around(x, y, 1);
                for (x, y) in vs {
                    if let Some(px) = self.at(x, y) {
                        if px.data[0] != 0 {
                            c = Some(px);
                            break;
                        }
                    }
                }

                let c = match c {
                    Some(x) => x,
                    None => continue,
                };

                let px = match self.at_mut(x, y) {
                    Some(x) => x,
                    None => continue,
                };

                *px = c.mutate(r*4);
            }
        }
    }

    fn save(&self, path: &Path) {
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut encoder = png::Encoder::new(w, self.width, self.height); // Width is 2 pixels and height is 1.
        encoder.set_compression(png::Compression::Fast);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&self.data).unwrap(); // Save
    }
}

fn main() {
    println!("Creating image");
    let mut img = Image::new(1920, 1080);
    println!("generating...");
    img.generate();
    println!("Saving...");
    img.save(Path::new("out.png"));
}
