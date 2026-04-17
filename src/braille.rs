use crate::{
    Color,
    utils::{write_bg_color, write_fg_color, write_move_to},
};

/// renders to strings using braille characters
///
/// - sets colors of dots
///
///   NOTE: color works on a "last-wins" strategy. the last color
///   set for a dot within a single terminal cell dictates the
///   color of every dot in the cell
///
/// - renders to a string that the caller can write to their screen
pub struct BrailleCanvas {
    /// (rows, columns) in *cells*
    dimensions: (usize, usize),

    /// (row_offset, col_offset) in *cells*
    offset: (usize, usize),

    /// data is per *dot*
    buffer: Vec<bool>,

    /// data is per *cell*
    colors: Vec<Option<Color>>,

    /// optional color used for the entire canvas background
    bg_color: Option<Color>,
}

impl BrailleCanvas {
    pub fn new(dimensions: (usize, usize), offset: (usize, usize)) -> BrailleCanvas {
        let (rows, cols) = dimensions;

        let buffer = vec![false; 8 * rows * cols];
        let colors = vec![None; rows * cols];

        BrailleCanvas {
            dimensions,
            offset,
            buffer,
            colors,
            bg_color: None,
        }
    }

    pub const fn width(&self) -> usize {
        2 * self.dimensions.1
    }

    pub const fn height(&self) -> usize {
        4 * self.dimensions.0
    }

    /// x and y are in canvas coordinates
    pub fn set(&mut self, x: usize, y: usize, color: Color) {
        if x >= self.width() || y >= self.height() {
            return;
        }

        let buffer_idx = y * self.width() + x;
        self.buffer[buffer_idx] = true;

        let cell_idx = (y / 4) * self.dimensions.1 + (x / 2);
        self.colors[cell_idx] = Some(color);
    }

    /// x and y are in canvas coordinates
    pub fn set_f(&mut self, x: f64, y: f64, color: Color) {
        self.set(x.round() as usize, y.round() as usize, color);
    }

    /// sets a background color for the entire canvas
    pub fn set_bg_color(&mut self, color: Color) {
        self.bg_color = Some(color);
    }

    fn clear_buffer(&mut self) {
        self.buffer.fill(false);
        self.colors.fill(None);
    }

    pub fn render(&mut self) -> String {
        let width = self.width();

        let mut out = String::new();

        if let Some(bg_color) = self.bg_color {
            write_bg_color(&mut out, bg_color);
        }

        let mut current_color = None;

        for row in 0..self.dimensions.0 {
            write_move_to(&mut out, row + self.offset.0, self.offset.1);
            for col in 0..self.dimensions.1 {
                // write a color byte if the color has changed in this cell
                let cell_color = self.colors[row * width / 2 + col];
                if cell_color != current_color
                    && let Some(cell_color) = cell_color
                {
                    write_fg_color(&mut out, cell_color);
                    current_color = Some(cell_color);
                }

                // figure out which dots are set inside this cell
                let mut byte = 0;

                // the braille unicode character 0x28XX puts dots based on the bits of the
                // 'XX' bytes, according to this layout:
                //
                // 0 3
                // 1 4
                // 2 5
                // 6 7   <- annoying bottom row
                //
                // so we and in 1's offset by appropriate amounts based on which cells
                // we want turned on
                if self.buffer[4 * row * width + 2 * col] {
                    byte |= 1;
                }
                if self.buffer[4 * row * width + 2 * col + 1] {
                    byte |= 1 << 3;
                }
                if self.buffer[(4 * row + 1) * width + 2 * col] {
                    byte |= 1 << 1;
                }
                if self.buffer[(4 * row + 1) * width + 2 * col + 1] {
                    byte |= 1 << 4;
                }
                if self.buffer[(4 * row + 2) * width + 2 * col] {
                    byte |= 1 << 2;
                }
                if self.buffer[(4 * row + 2) * width + 2 * col + 1] {
                    byte |= 1 << 5;
                }
                if self.buffer[(4 * row + 3) * width + 2 * col] {
                    byte |= 1 << 6;
                }
                if self.buffer[(4 * row + 3) * width + 2 * col + 1] {
                    byte |= 1 << 7;
                }

                if byte == 0 {
                    out.push(' ');
                } else {
                    out.push(char::from_u32(0x2800 | byte as u32).unwrap());
                }
            }
        }

        // clear to prepare the next render
        self.clear_buffer();

        out
    }
}
