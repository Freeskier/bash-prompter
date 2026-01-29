#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    x: u16,
    y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }

    pub fn x(&self) -> u16 {
        self.x
    }

    pub fn y(&self) -> u16 {
        self.y
    }

    pub fn set(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }
}
