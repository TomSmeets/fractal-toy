use crate::math::*;

pub fn draw_tile(pixels: &mut [u8], x: i64, y: i64, z: i32) {
	let resolution: u32 = 256;
	assert!(pixels.len() as u32 == resolution * resolution * 4);

	let scale = 0.5f64.powi(z - 2);
	let center = Vector2::new(x as f64, y as f64) * scale - Vector2::new(2.0, 2.0);
	draw_mandel(pixels, resolution, resolution, scale, center);
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

			let color = (mandel(256, c0) * 255 / 256) as u8;

			pixels[(0 + (x + y * w) * 4) as usize] = 255;
			pixels[(1 + (x + y * w) * 4) as usize] = color;
			pixels[(2 + (x + y * w) * 4) as usize] = color;
			pixels[(3 + (x + y * w) * 4) as usize] = color;
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
