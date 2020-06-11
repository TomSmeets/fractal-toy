mod map;
mod pos;

// TODO: rename to 'TileMap'
pub use self::map::TileStorage;
pub use self::pos::TilePos;

// TODO: remove, this does not belong here
mod content;
pub use self::content::TileContent;
pub use self::map::Task;
