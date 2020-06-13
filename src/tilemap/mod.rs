mod map;
mod pos;
mod square;

// TODO: rename to 'TileMap'
pub use self::map::TileStorage;
pub use self::pos::TilePos;
pub use self::square::Square;

// TODO: remove, this does not belong here
pub use self::map::Task;
