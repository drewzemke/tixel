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

#[cfg(test)]
mod tests {
    use super::*;

    // strips ANSI escape sequences, returning just the visible characters
    fn visible_chars(output: &str) -> Vec<char> {
        let mut chars = Vec::new();
        let mut in_escape = false;
        for c in output.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape {
                if c.is_ascii_alphabetic() {
                    in_escape = false;
                }
            } else {
                chars.push(c);
            }
        }
        chars
    }

    const WHITE: Color = Color::new(255, 255, 255);

    #[test]
    fn empty_canvas_renders_spaces() {
        let mut canvas = BrailleCanvas::new((2, 3), (0, 0));
        let output = canvas.render();
        let chars = visible_chars(&output);
        assert_eq!(chars.len(), 6);
        assert!(chars.iter().all(|&c| c == ' '));
    }

    #[test]
    fn each_dot_maps_to_correct_braille_bit() {
        // dot positions within a cell and their expected braille bits:
        //   (0,0)->0  (1,0)->3
        //   (0,1)->1  (1,1)->4
        //   (0,2)->2  (1,2)->5
        //   (0,3)->6  (1,3)->7
        let expected: [(usize, usize, u32); 8] = [
            (0, 0, 0x01),
            (1, 0, 0x08),
            (0, 1, 0x02),
            (1, 1, 0x10),
            (0, 2, 0x04),
            (1, 2, 0x20),
            (0, 3, 0x40),
            (1, 3, 0x80),
        ];

        for (x, y, bit) in expected {
            let mut canvas = BrailleCanvas::new((1, 1), (0, 0));
            canvas.set(x, y, WHITE);
            let output = canvas.render();
            let chars = visible_chars(&output);
            assert_eq!(
                chars[0],
                char::from_u32(0x2800 | bit).unwrap(),
                "dot ({x}, {y}) should set bit {bit:#04x}"
            );
        }
    }

    #[test]
    fn set_out_of_bounds_does_not_panic() {
        let mut canvas = BrailleCanvas::new((1, 1), (0, 0));
        canvas.set(2, 0, WHITE);
        canvas.set(0, 4, WHITE);
        canvas.set(100, 100, WHITE);
        let chars = visible_chars(&canvas.render());
        assert!(chars.iter().all(|&c| c == ' '));
    }

    #[test]
    fn render_clears_buffer() {
        let mut canvas = BrailleCanvas::new((1, 1), (0, 0));
        canvas.set(0, 0, WHITE);
        let _ = canvas.render();

        // second render without any new sets should be empty
        let chars = visible_chars(&canvas.render());
        assert!(chars.iter().all(|&c| c == ' '));
    }

    #[test]
    fn multiple_dots_combine_in_one_cell() {
        let mut canvas = BrailleCanvas::new((1, 1), (0, 0));
        // top-left and bottom-right dots
        canvas.set(0, 0, WHITE); // bit 0
        canvas.set(1, 3, WHITE); // bit 7
        let chars = visible_chars(&canvas.render());
        assert_eq!(chars[0], char::from_u32(0x2800 | 0x01 | 0x80).unwrap());
    }
}
