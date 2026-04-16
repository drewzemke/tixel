#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }

    pub const fn r(&self) -> u8 {
        self.0
    }

    pub const fn g(&self) -> u8 {
        self.1
    }

    pub const fn b(&self) -> u8 {
        self.2
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}
