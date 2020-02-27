use crate::math::*;
use crate::quadtree::pos::*;
use ::palette::*;

pub fn draw_tile(pixels: &mut [u8], p: QuadTreePosition) {
	let resolution: u32 = 256;
	// TODO: improve
	assert!(pixels.len() as u32 == resolution * resolution * 4);

	// gets center of this qpos square
	let (x, y, size) = p.float_top_left_with_size();
	let center = Vector2::new(x, y) * 4.0 - Vector2::new(2.0, 2.0);
	draw_mandel(pixels, resolution, resolution, size * 4.0, center);
}

fn draw_mandel(pixels: &mut [u8], w: u32, h: u32, zoom: f64, offset: Vector2<f64>) {
	for y in 0..h {
		for x in 0..w {
			let mut c0 = Vector2::new(x as f64, y as f64);

			// screen coords 0 - 1
			c0.x /= w as f64;
			c0.y /= h as f64;

			// -1 , 1
			c0 = zoom * c0 + offset;

            let itr = mandel(256, c0);

            let mut v = itr as f32 / 256.0;
            v *= v;
            v = 1. - v;

			let hsv = Hsv::new(itr as f32 / 32.0 * 360., v, v);
			let rgb = Srgb::from(hsv).into_linear();

			pixels[(0 + (x + y * w) * 4) as usize] = 255;
			pixels[(1 + (x + y * w) * 4) as usize] = (rgb.red * 255.) as u8;
			pixels[(2 + (x + y * w) * 4) as usize] = (rgb.green * 255.) as u8;
			pixels[(3 + (x + y * w) * 4) as usize] = (rgb.blue * 255.) as u8;
		}
	}
}

fn mandel(max: i32, c: Vector2<f64>) -> i32 {
	let mut z = c;

	let mut n = 0;
	loop {
		let r = z.x;
		let i = z.y;
		z.x = r * r - i * i + c.x;
		z.y = 2.0 * r * i + c.y;

		if r * r + i * i > 4.0 {
			return n;
		}

		if n == max {
			return max;
		}
		n += 1;
	}
}
