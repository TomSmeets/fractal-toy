use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use image::bmp::BMPEncoder;
use image::ColorType;
use rand::prelude::*;
use rand::rngs::SmallRng;

#[derive(Clone)]
struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    fn mutate(&self, gen: &mut impl Rng) -> Self {
        let l = 0.0082;
        let mut c = self.clone();
        c.r += gen.gen::<f32>() * 2.0 * l - l;
        c.g += gen.gen::<f32>() * 2.0 * l - l;
        c.b += gen.gen::<f32>() * 2.0 * l - l;

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

impl Image {
    fn new(width: u32, height: u32) -> Self {
        Image {
            width,
            height,
            data: vec![None; (width * height) as usize],
        }
    }

    fn check(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.width as i32 && y < self.height as i32
    }

    fn around(&self, i: i32, j: i32, r: i32, gen: &mut impl Rng) -> Vec<(i32, i32)> {
        let mut xs = Vec::new();

        let put = |xs: &mut Vec<(i32, i32)>, x, y| {
            if self.check(x, y) {
                xs.push((x, y));
            }
        };

        for o in -r..=r {
            put(&mut xs, i + o, j + r);
            put(&mut xs, i + o, j - r);
            put(&mut xs, i + r, j + o);
            put(&mut xs, i - r, j + o);
        }

        // TODO: Shuffle

        xs.shuffle(gen);
        xs
    }

    fn at_mut(&mut self, x: i32, y: i32) -> Option<&mut Option<Color>> {
        if !self.check(x, y) {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            Some(&mut self.data[i])
        }
    }

    fn at(&self, x: i32, y: i32) -> Option<Option<Color>> {
        if !self.check(x, y) {
            None
        } else {
            let i = (y as u32 * self.width + x as u32) as usize;
            Some(self.data[i].clone())
        }
    }

    fn generate(&mut self, gen: &mut impl Rng) {
        // center
        let cx = gen.gen_range(0, self.width as i32);
        let cy = gen.gen_range(0, self.height as i32);
        let ring_count = *[cx, cy, self.width as i32 - cx, self.height as i32 - cy]
            .iter()
            .max()
            .unwrap_or(&0);

        {
            let r = gen.gen::<f32>();
            let g = gen.gen::<f32>();
            let b = gen.gen::<f32>();

            let l = (r * r + g * g + b * b).sqrt();

            let p = self.at_mut(cx, cy).unwrap();
            *p = Some(Color {
                r: r / l,
                g: g / l,
                b: b / l,
            });
        }

        let mut p_old = 0;
        for r in 1..ring_count {
            {
                let p = r * 100 / ring_count;
                if p != p_old {
                    println!("progress: {}%", p);
                    p_old = p;
                }
            }
            let vs = self.around(cx, cy, r, gen);
            for (x, y) in vs {
                let mut c: Option<Color> = None;
                for (x, y) in self.around(x, y, 1, gen) {
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

                *px = Some(c.mutate(gen));
            }
        }
    }

    fn save(&self, path: &Path) {
        let file = File::create(path).unwrap();
        let mut w = BufWriter::new(file);

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

        let mut enc = BMPEncoder::new(&mut w);
        enc.encode(&data, self.width, self.height, ColorType::Rgb8)
            .unwrap();
    }
}

fn x11_resolution() -> (u32, u32) {
    let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
    let setup = conn.get_setup();
    let screen = setup.roots().nth(screen_num as usize).unwrap();
    (
        screen.width_in_pixels() as u32,
        screen.height_in_pixels() as u32,
    )
}

#[test]
fn test_small_image() {
    let mut gen = SmallRng::seed_from_u64(0);
    let mut img = Image::new(64, 64);
    img.generate(&mut gen);
}

#[test]
fn test_1x1() {
    let mut gen = SmallRng::seed_from_u64(0);
    let mut img = Image::new(1, 1);
    img.generate(&mut gen);
}

fn main() {
    let mut gen = SmallRng::seed_from_u64(1);
    let res = x11_resolution();
    println!("resolution: {} x {}", res.0, res.1);
    println!("Creating image");
    let mut img = Image::new(res.0, res.1);
    println!("generating...");
    img.generate(&mut gen);
    println!("Saving...");
    img.save(Path::new("target/out.bmp"));
}
