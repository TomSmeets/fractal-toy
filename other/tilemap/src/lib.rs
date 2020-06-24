mod map;
mod pos;
mod square;
mod compare_iter;

pub use self::map::TileMap;
pub use self::pos::TilePos;
pub use self::square::Square;

// TODO: remove, this does not belong here
pub use self::map::Task;
