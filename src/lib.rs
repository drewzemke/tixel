use std::fmt::Write;

const UPPER_HALF_CELL: char = '▀';

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Color(u8, u8, u8);

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self(r, g, b)
    }
}

type Buffer = Vec<Option<Color>>;

/// renders to strings using the half block character
/// - allows setting color values
/// - renders to a string that the caller can write to their screen
pub struct HalfCellCanvas {
    /// (rows, columns) in *cells*
    dimensions: (usize, usize),

    /// (row_offset, col_offset) in *cells*
    offset: (usize, usize),

    buffers: [Buffer; 2],
    front_idx: usize,
}

/// writes a move-to escape seq to a string buffer. NOTE: row and col are *ZERO*-based
pub fn write_move_to(str: &mut String, row: usize, col: usize) {
    let _ = write!(str, "\x1b[{};{}H", row + 1, col + 1);
}

pub fn write_fg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[38;2;{};{};{}m", color.0, color.1, color.2);
}

pub fn write_bg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[48;2;{};{};{}m", color.0, color.1, color.2);
}

pub fn write_fg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[39m",);
}

pub fn write_bg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[49m",);
}

impl HalfCellCanvas {
    pub fn new(dimensions: (usize, usize), offset: (usize, usize)) -> Self {
        let (rows, cols) = dimensions;

        let pixels = vec![None; 2 * rows * cols];
        let buffers = [pixels.clone(), pixels];

        Self {
            dimensions,
            offset,
            buffers,
            front_idx: 0,
        }
    }

    pub fn width(&self) -> usize {
        self.dimensions.1
    }

    pub fn height(&self) -> usize {
        2 * self.dimensions.0
    }

    /// returns (front, back)
    fn buffers(&mut self) -> (&Buffer, &mut Buffer) {
        let [front, back] = self
            .buffers
            .get_disjoint_mut([self.front_idx, 1 - self.front_idx])
            .unwrap();
        (front, back)
    }

    fn swap_buffers(&mut self) {
        self.front_idx = 1 - self.front_idx;
    }

    fn clear_back_buffer(&mut self) {
        let (_, back) = self.buffers();
        back.fill(None);
    }

    /// x and y are in canvas space, not terminal space
    /// x is distance from left edge, y is distance from top
    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        let idx = y * self.width() + x;
        let (_, back) = self.buffers();
        back[idx] = Some(color)
    }

    /// Resets the internal buffers, guaranteeing a full-redraw on the
    /// next render
    pub fn reset(&mut self) {
        self.clear_back_buffer();
        self.swap_buffers();
        self.clear_back_buffer();
    }

    pub fn render(&mut self) -> String {
        // NOTE: estimating 40 bytes worse case for a foreground+background+half-cell output
        let mut out = String::with_capacity(self.width() * self.height() * 40);

        let (row_offset, col_offset) = self.offset;
        let width = self.width();

        let mut current_top: Option<Color> = None;
        let mut current_bottom: Option<Color> = None;

        let (rows, cols) = self.dimensions;

        let (front, back) = self.buffers();

        let mut skipping;

        for row in 0..rows {
            write_move_to(&mut out, row_offset + row, col_offset);
            skipping = true;

            for col in 0..cols {
                let back_top = back[(2 * row) * width + col];
                let back_bottom = back[(2 * row + 1) * width + col];

                // compare to front. if it's the same, skip
                let front_top = front[(2 * row) * width + col];
                let front_bottom = front[(2 * row + 1) * width + col];
                if front_top == back_top && front_bottom == back_bottom {
                    skipping = true;
                    continue;
                }

                // emit a move-to seq before writing if we've previously skipped some cells
                if skipping {
                    skipping = false;
                    write_move_to(&mut out, row_offset + row, col_offset + col);
                }

                if let Some(top_color) = back_top {
                    if current_top.is_none_or(|c| c != top_color) {
                        write_fg_color(&mut out, top_color);
                        current_top = Some(top_color);
                    };
                } else if current_top.is_some() {
                    write_fg_reset(&mut out);
                    current_top = None;
                };

                if let Some(bottom_color) = back_bottom {
                    if current_bottom.is_none_or(|c| c != bottom_color) {
                        write_bg_color(&mut out, bottom_color);
                        current_bottom = Some(bottom_color);
                    }
                } else if current_bottom.is_some() {
                    write_bg_reset(&mut out);
                    current_bottom = None;
                };

                let _ = write!(&mut out, "{UPPER_HALF_CELL}");
            }
        }

        self.swap_buffers();
        self.clear_back_buffer();

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_only_outputs_changed_pixels() {
        let mut canvas = HalfCellCanvas::new((1, 6), (0, 0));

        // fill the canvas
        for x in 0..canvas.width() {
            canvas.set_color(x, 0, Color::new(0, 0, 0));
        }

        // render
        let _ = canvas.render();

        // fill canvas again; changing the first and last pixel
        for x in 1..canvas.width() - 1 {
            canvas.set_color(x, 0, Color::new(0, 0, 0));
        }
        canvas.set_color(0, 0, Color::new(100, 100, 100));
        canvas.set_color(canvas.width() - 1, 0, Color::new(200, 200, 200));

        // render again and look for a "move" escape seq
        let output = canvas.render();
        assert!(output.contains(&format!("\x1b[{};{}H", 1, canvas.width())));
    }
}
