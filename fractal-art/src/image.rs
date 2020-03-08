use image::bmp::BMPEncoder;
use image::ColorType;
use rand::prelude::*;

use crate::color::Color;

pub struct Image {
    width: u32,
    height: u32,
    data: Vec<Option<Color>>,
}

impl Image {
    pub fn new(width: u32, height: u32) -> Self {
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

    pub fn generate(&mut self, gen: &mut impl Rng, (cx, cy): (u32, u32)) -> Result<(), String> {
        let cx = cx as i32;
        let cy = cy as i32;

        // center
        let ring_count = *[cx, cy, self.width as i32 - cx, self.height as i32 - cy]
            .iter()
            .max()
            .unwrap_or(&0);

        {
            let r = gen.gen::<f32>();
            let g = gen.gen::<f32>();
            let b = gen.gen::<f32>();

            let l = (0.299 * r * r + 0.587 * g * g + 0.114 * b * b).sqrt();

            let p = self.at_mut(cx, cy).expect("Center point is out of range!");
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
                    eprintln!("progress: {}%", p);
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
        Ok(())
    }

    pub fn save(&self, writer: &mut impl std::io::Write) {
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

        let mut enc = BMPEncoder::new(writer);
        enc.encode(&data, self.width, self.height, ColorType::Rgb8)
            .unwrap();
    }
}
