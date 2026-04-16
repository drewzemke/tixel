use crate::utils::write_move_to;

type Buffer = Vec<bool>;

/// renders to strings using the half block character
/// - allows setting color values
/// - renders to a string that the caller can write to their screen
pub struct BrailleCanvas {
    /// (rows, columns) in *cells*
    dimensions: (usize, usize),

    /// (row_offset, col_offset) in *cells*
    offset: (usize, usize),

    buffer: Buffer,
}

impl BrailleCanvas {
    pub fn new(dimensions: (usize, usize), offset: (usize, usize)) -> BrailleCanvas {
        let (rows, cols) = dimensions;

        let buffer = vec![false; 8 * rows * cols];

        BrailleCanvas {
            dimensions,
            offset,
            buffer,
        }
    }

    pub const fn width(&self) -> usize {
        2 * self.dimensions.1
    }

    pub const fn height(&self) -> usize {
        4 * self.dimensions.0
    }

    // FIXME what should we do if x or y are out of bounds?
    /// x and y are in canvas coordinates
    pub fn set(&mut self, x: usize, y: usize) {
        if x >= self.width() || y >= self.height() {
            return;
        }

        let idx = y * self.width() + x;
        self.buffer[idx] = true;
    }

    /// x and y are in canvas coordinates
    pub fn set_f(&mut self, x: f64, y: f64) {
        self.set(x.round() as usize, y.round() as usize);
    }

    fn clear_buffer(&mut self) {
        self.buffer.fill(false);
    }

    pub fn render(&mut self) -> String {
        let mut out = String::new();
        write_move_to(&mut out, 0, 0);

        let width = self.width();

        for row in 0..self.dimensions.0 {
            for col in 0..self.dimensions.1 {
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

        // clear to get read for the next render
        self.clear_buffer();

        out
    }
}
