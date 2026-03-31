use std::fmt::Write;

const UPPER_HALF_CELL: char = '▀';

#[derive(Debug, Default, Clone, Copy)]
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
        out.push_str(&format!("\x1b[{};{}H", 0, 0));

        for row in 0..self.terminal_rows {
            for col in 0..self.terminal_cols {
                let top_color = self.pixels[2 * row][col];
                let bottom_color = self.pixels[2 * row + 1][col];

                if let Some(top_color) = top_color {
                    let _ = write!(
                        &mut out,
                        "\x1b[38;2;{};{};{}m",
                        top_color.0, top_color.1, top_color.2
                    );
                };

                if let Some(bottom_color) = bottom_color {
                    let _ = write!(
                        &mut out,
                        "\x1b[48;2;{};{};{}m",
                        bottom_color.0, bottom_color.1, bottom_color.2
                    );
                };

                let _ = write!(&mut out, "{UPPER_HALF_CELL}");
            }
        }

        out
    }
}
