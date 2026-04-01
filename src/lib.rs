use std::fmt::Write;

const UPPER_HALF_CELL: char = '▀';

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }
}

/// renders to strings using the half block character
/// - allows setting color values
/// - renders to a string that the caller can write to their screen
pub struct HalfCellCanvas {
    terminal_rows: usize,
    terminal_cols: usize,
    pixels: Vec<Vec<Option<Color>>>,
}

/// writes a move-to escape seq to a string buffer. NOTE: row and col are *ZERO*-based
fn write_move_to(str: &mut String, row: usize, col: usize) {
    let _ = write!(str, "\x1b[{};{}H", row + 1, col + 1);
}

fn write_fg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[38;2;{};{};{}m", color.0, color.1, color.2);
}

fn write_bg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[48;2;{};{};{}m", color.0, color.1, color.2);
}

fn write_fg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[39m",);
}

fn write_bg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[49m",);
}

impl HalfCellCanvas {
    pub fn new(terminal_rows: usize, terminal_cols: usize) -> Self {
        let pixels = vec![vec![None; terminal_cols]; 2 * terminal_rows];

        Self {
            terminal_rows,
            terminal_cols,
            pixels,
        }
    }

    pub fn width(&self) -> usize {
        self.terminal_cols
    }

    pub fn height(&self) -> usize {
        2 * self.terminal_rows
    }

    /// x and y are in canvas space, not terminal space
    /// x is distance from left edge, y is distance from top
    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        self.pixels[y][x] = Some(color)
    }

    pub fn render(&self) -> String {
        // NOTE: estimating 40 bytes worse case for a foreground+background+half-cell output
        let mut out = String::with_capacity(self.width() * self.height() * 40);
        write_move_to(&mut out, 0, 0);

        let mut current_top: Option<Color> = None;
        let mut current_bottom: Option<Color> = None;

        for row in 0..self.terminal_rows {
            for col in 0..self.terminal_cols {
                let top_color = self.pixels[2 * row][col];
                let bottom_color = self.pixels[2 * row + 1][col];

                if let Some(top_color) = top_color {
                    if current_top.is_none_or(|c| c != top_color) {
                        write_fg_color(&mut out, top_color);
                        current_top = Some(top_color);
                    };
                } else {
                    if current_top.is_some() {
                        write_fg_reset(&mut out);
                        current_top = None;
                    }
                };

                if let Some(bottom_color) = bottom_color {
                    if current_bottom.is_none_or(|c| c != bottom_color) {
                        write_bg_color(&mut out, bottom_color);
                        current_bottom = Some(bottom_color);
                    }
                } else {
                    if current_bottom.is_some() {
                        write_bg_reset(&mut out);
                        current_bottom = None;
                    }
                };

                let _ = write!(&mut out, "{UPPER_HALF_CELL}");
            }
        }

        out
    }
}
