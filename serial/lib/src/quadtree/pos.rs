#[derive(Debug, PartialEq, Eq, Clone)]
pub struct QuadTreePosition {
	pub path: Vec<u8>,
}

impl QuadTreePosition {
	pub fn from(path: &[u8]) -> QuadTreePosition {
		QuadTreePosition {
			path: path.to_vec(),
		}
	}

	pub fn depth(&self) -> u32 {
		self.path.len() as u32
	}

	pub fn root() -> Self {
		QuadTreePosition { path: Vec::new() }
	}

	pub fn parent(&mut self) {
		self.path.pop();
	}

	pub fn child(&mut self, x: u8, y: u8) {
		self.path.push(y*2 + x);
	}

	pub fn float_top_left_with_size(&self) -> (f64, f64, f64) {
		let mut s = 1.0;
		let mut x = 0.0;
		let mut y = 0.0;
		for i in &self.path {
			s *= 0.5;
			if *i % 2 == 1 {
				x += s;
			}
			if *i / 2 == 1 {
				y += s;
			}
		}
		(x, y, s)
	}

	// let (x, y) = p.float_top_left();
	// let size = p.float_size();
}
