pub mod helper;
pub mod post;

pub struct Data {
    x: f32,
    y: f32,
    len: u32,
}

impl Data {
    pub fn new(x: f32, y: f32, len: u32) -> Self {
        Data { x, y, len }
    }
}
