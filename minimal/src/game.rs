pub trait Game {
    fn init() -> Self;
    fn update(&mut self);
}
