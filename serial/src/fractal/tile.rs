mod content;
mod pos;

pub use self::content::TileContent;
pub use self::pos::TilePos;

pub enum TileState {
    Queued,
    Working,
    Done(TileContent),
}
