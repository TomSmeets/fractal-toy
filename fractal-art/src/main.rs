use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::bmp::BMPEncoder;
use image::ColorType;
use rand::prelude::*;

// TODO: use image crate instad of png, and save as bmp

#[derive(Clone)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    fn mutate(&self) -> Self {
        let l = 0.0081;
        let mut c = self.clone();
        c.r += rand::random::<f32>() * 2.0 * l - l;
        c.g += rand::random::<f32>() * 2.0 * l - l;
        c.b += rand::random::<f32>() * 2.0 * l - l;

        fn clamp(x: &mut f32) {
            if *x > 1.0 {
                *x = 1.0;
            } else if *x < 0.0 {
                *x = 0.0;
            };
        }

        clamp(&mut c.r);
        clamp(&mut c.g);
        clamp(&mut c.b);
        c
    }
}

struct Image {
    width: u32,
    height: u32,
    data: Vec<Option<Color>>,
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
            data: vec![None; (width * height) as usize],
        }
    }

    fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Option<Color>> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            Some(&mut self.data[i])
        }
    }

    fn at(&self, x: i32, y: i32) -> Option<Option<Color>> {
        if x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            Some(self.data[i].clone())
        }
    }

    fn generate(&mut self) {
        // center
        let cx = (self.width / 2) as i32;
        let cy = (self.height / 2) as i32;
        let ring_count = *[ cx, cy, self.width as i32 - cx, self.height as i32 - cy ].iter().max().unwrap_or(&0);

        {
            let p = self.at_mut(cx, cy).unwrap();
            *p = Some(Color {
                r: 1.0,
                g: 0.0,
                b: 0.0,
            });
        }

        for r in 1..ring_count {
            println!("i = {}", r);
            let vs = around(cx, cy, r);
            for (x, y) in vs {
                let mut c: Option<Color> = None;
                for (x, y) in around(x, y, 1) {
                    if let Some(Some(px)) = self.at(x, y) {
                        c = Some(px);
                        break;
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

                *px = Some(c.mutate());
            }
        }
    }

    fn save(&self, path: &Path) {
        let file = File::create(path).unwrap();
        let ref mut w = BufWriter::new(file);

        let mut data = Vec::with_capacity(self.data.len() * 3);
        for c in self.data.iter() {
            fn to_u8(x: f32) -> u8 {
                if x < 0.0 {
                    return 0;
                }
                if x > 1.0 {
                    return 255;
                }
                (x * 255.0) as u8
            }

            let c_default = Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            };
            let c = match c {
                Some(x) => &x,
                None => &c_default,
            };
            data.push(to_u8(c.r));
            data.push(to_u8(c.g));
            data.push(to_u8(c.b));
        }

        let mut enc = BMPEncoder::new(w);
        enc.encode(&data, self.width, self.height, ColorType::Rgb8)
            .unwrap();
    }
}

#[test]
fn test_bench() {
    let mut img = Image::new(2048, 2048);
    img.generate();
}

fn main() {
    println!("Creating image");
    let mut img = Image::new(64, 64);
    println!("generating...");
    img.generate();
    println!("Saving...");
    img.save(Path::new("out.png"));
}
