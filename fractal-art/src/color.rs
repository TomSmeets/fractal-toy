use rand::prelude::*;

#[derive(Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub fn mutate(&self, gen: &mut impl Rng) -> Self {
        let l = 0.0060;
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
