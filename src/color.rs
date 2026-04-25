use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Ansi(AnsiColor),
    Rgb(u8, u8, u8),
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        Self::Rgb(value.0, value.1, value.2)
    }
}

impl Color {
    pub fn write_fg(&self, str: &mut String) {
        let _ = match self {
            Color::Ansi(ansi_color) => write!(str, "\x1b[{}m", ansi_color.fg_code()),
            Color::Rgb(r, g, b) => write!(str, "\x1b[38;2;{};{};{}m", r, g, b),
        };
    }

    pub fn write_bg(&self, str: &mut String) {
        let _ = match self {
            Color::Ansi(ansi_color) => write!(str, "\x1b[{}m", ansi_color.bg_code()),
            Color::Rgb(r, g, b) => write!(str, "\x1b[48;2;{};{};{}m", r, g, b),
        };
    }

    pub fn escape_fg(&self) -> String {
        let mut str = String::new();
        self.write_fg(&mut str);
        str
    }

    pub fn escape_bg(&self) -> String {
        let mut str = String::new();
        self.write_bg(&mut str);
        str
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnsiColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl AnsiColor {
    const fn fg_code(&self) -> u8 {
        match self {
            AnsiColor::Black => 30,
            AnsiColor::Red => 31,
            AnsiColor::Green => 32,
            AnsiColor::Yellow => 33,
            AnsiColor::Blue => 34,
            AnsiColor::Magenta => 35,
            AnsiColor::Cyan => 36,
            AnsiColor::White => 37,
            AnsiColor::BrightBlack => 90,
            AnsiColor::BrightRed => 91,
            AnsiColor::BrightGreen => 92,
            AnsiColor::BrightYellow => 93,
            AnsiColor::BrightBlue => 94,
            AnsiColor::BrightMagenta => 95,
            AnsiColor::BrightCyan => 96,
            AnsiColor::BrightWhite => 97,
        }
    }

    const fn bg_code(&self) -> u8 {
        self.fg_code() + 10
    }
}
