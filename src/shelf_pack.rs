use crate::util::Rect;
use crate::util::V2;

struct Shelf {
    width: u32,
    height: u32,
}

/// Write only rect packer
pub struct ShelfPack {
    size: V2<u32>,
    shelves: Vec<Shelf>,
}

impl ShelfPack {
    pub fn new(size: V2<u32>) -> Self {
        ShelfPack {
            size,
            shelves: Vec::new(),
        }
    }

    pub fn add(&mut self, image_size: V2<u32>) -> Option<Rect> {
        let pad = 1;
        let image_size = image_size + V2::new(pad, pad);
        // TODO: implement rotation?

        let mut best_shelf = None;

        let mut y = 0;
        for (shelf_index, shelf) in self.shelves.iter().enumerate() {
            let fits_x = shelf.width + image_size.x <= self.size.x;
            let fits_y = image_size.y <= shelf.height;

            // skip this shelf if we don't fit
            if fits_x && fits_y {
                let error = shelf.height - image_size.y;

                let is_better = match best_shelf {
                    Some((best_error, _, _)) => best_error > error,
                    None => true,
                };

                if is_better {
                    best_shelf = Some((error, shelf_index, y));
                }
            }

            y += shelf.height;
        }

        let (ix, y) = match best_shelf {
            Some((_, ix, y)) => (ix, y),
            None => {
                let fits_x = image_size.x <= self.size.x;
                let fits_y = y + image_size.y <= self.size.y;

                if !fits_x || !fits_y {
                    return None;
                }

                let ix = self.shelves.len();
                self.shelves.push(Shelf {
                    width: 0,
                    height: image_size.y,
                });
                (ix, y)
            }
        };

        let shelf = &mut self.shelves[ix];
        let x = shelf.width;
        shelf.width += image_size.x;

        Some(Rect::corner_size(
            V2::new(x as _, y as _),
            V2::new((image_size.x - pad) as _, (image_size.y - pad) as _),
        ))
    }
}
