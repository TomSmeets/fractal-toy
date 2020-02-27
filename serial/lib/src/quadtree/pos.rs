#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct QuadTreePosition {
	pub x: u64,
	pub y: u64,
	pub z: u64,
}

impl QuadTreePosition {
	fn new(x: u64, y: u64, z: u64) -> Self {
		let q = QuadTreePosition { x, y, z };
		q.check();
		q
	}

	fn check(&self) {
		let dim = self.dim();
		assert!(self.x <= dim);
		assert!(self.y <= dim);
	}

	pub fn dim(&self) -> u64 {
		if self.z == 0 {
			0
		} else {
			1 << (self.z - 1)
		}
	}

	fn root() -> Self {
		Self::new(0, 0, 0)
	}

	fn parent(&self) -> Self {
		QuadTreePosition {
			x: self.x >> 1,
			y: self.y >> 1,
			z: self.z - 1,
		}
	}

	fn child(&self, qx: u8, qy: u8) -> Self {
		QuadTreePosition {
			x: (self.x << 1) + qx as u64,
			y: (self.y << 1) + qy as u64,
			z: self.z + 1,
		}
	}
}

#[test]
fn test_pos() {
	let p = QuadTreePosition::root();

	{
		let q = p.child(0, 0).child(0, 0).child(0, 0);
		assert_eq!(q.z, 3);
		assert_eq!(q.x, 0);
		assert_eq!(q.y, 0);
	}

	{
		let q = p.child(1, 1);
		assert_eq!(q.z, 1);
		assert_eq!(q.x, 1);
		assert_eq!(q.y, 1);
		assert_eq!(q.parent(), p);
	}

	{
		let q = p.child(1, 1).child(1, 1);
		assert_eq!(q.z, 2);
		assert_eq!(q.x, 3);
		assert_eq!(q.y, 3);
		assert_eq!(q.parent().parent(), p);
	}

	{
		let q = p.child(1, 1).child(1, 1).child(1, 1);
		assert_eq!(q.z, 3);
		assert_eq!(q.x, 7);
		assert_eq!(q.y, 7);
		assert_eq!(q.parent().parent().parent(), p);
	}
}
