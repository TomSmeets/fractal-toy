pub struct Array2<T> {
    pub size_x: i32,
    pub size_y: i32,
    pub data: Vec<T>,
}

impl<T: Clone> Array2<T> {
    pub fn new(sx: u32, sy: u32, empty: T) -> Self {
        Array2 {
            size_x: sx as i32,
            size_y: sy as i32,
            data: vec![empty; sx as usize * sy as usize],
        }
    }

    pub fn is_inside(&self, (x, y): (i32, i32)) -> bool {
        !(x < 0 || y < 0 || x >= self.size_x || y >= self.size_y)
    }

    pub fn at(&self, (x, y): (i32, i32)) -> Option<&T> {
        if self.is_inside((x, y)) {
            Some(unsafe { self.data.get_unchecked((y * self.size_x + x) as usize) })
        } else {
            None
        }
    }

    pub fn set(&mut self, (x, y): (i32, i32), t: T) {
        if self.is_inside((x, y)) {
            self.data[(y * self.size_x + x) as usize] = t;
        }
    }
}
