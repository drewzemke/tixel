use std::fmt::Write;

use crate::utils::{write_bg_color, write_bg_reset, write_fg_color, write_fg_reset, write_move_to};

pub use crate::color::Color;

const UPPER_HALF: char = '▀';
const LOWER_HALF: char = '▄';

type Buffer = Vec<Option<Color>>;

/// renders to strings using the half block character
/// - allows setting color values
/// - renders to a string that the caller can write to their screen
pub struct HalfCellCanvas {
    /// (cols, rows) in *cells*
    dimensions: (usize, usize),

    /// (col_offset, row_offset) in *cells*
    offset: (usize, usize),

    buffers: [Buffer; 2],
    front_idx: usize,

    /// tracks the terminal's current fg/bg color state across renders
    current_top: Option<Color>,
    current_bottom: Option<Color>,
}

impl HalfCellCanvas {
    pub fn new(dimensions: (usize, usize), offset: (usize, usize)) -> Self {
        let (cols, rows) = dimensions;

        let pixels = vec![None; 2 * rows * cols];
        let buffers = [pixels.clone(), pixels];

        Self {
            dimensions,
            offset,
            buffers,
            front_idx: 0,
            current_top: None,
            current_bottom: None,
        }
    }

    pub fn width(&self) -> usize {
        self.dimensions.0
    }

    pub fn height(&self) -> usize {
        2 * self.dimensions.1
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
    ///
    /// ignores out-of-bounds input
    pub fn set_color(&mut self, x: usize, y: usize, color: Color) {
        if x > self.dimensions.0 || y > 2 * self.dimensions.1 {
            return;
        }

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
        self.current_top = None;
        self.current_bottom = None;
    }

    pub fn render_to(&mut self, buf: &mut String) {
        let (col_offset, row_offset) = self.offset;
        let width = self.width();

        let mut current_top = self.current_top;
        let mut current_bottom = self.current_bottom;

        let (cols, rows) = self.dimensions;

        let (front, back) = self.buffers();

        let mut skipping;

        for row in 0..rows {
            write_move_to(buf, col_offset, row_offset + row);
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
                    write_move_to(buf, col_offset + col, row_offset + row);
                }

                let desired_fg = back_top.or(back_bottom);
                let desired_bg = back_top.and(back_bottom);
                let ch = if back_top.is_some() {
                    UPPER_HALF
                } else if back_bottom.is_some() {
                    LOWER_HALF
                } else {
                    ' '
                };

                match desired_fg {
                    Some(fg) if current_top.is_none_or(|c| c != fg) => {
                        write_fg_color(buf, fg);
                        current_top = Some(fg);
                    }
                    None if current_top.is_some() => {
                        write_fg_reset(buf);
                        current_top = None;
                    }
                    _ => {}
                }
                match desired_bg {
                    Some(bg) if current_bottom.is_none_or(|c| c != bg) => {
                        write_bg_color(buf, bg);
                        current_bottom = Some(bg);
                    }
                    None if current_bottom.is_some() => {
                        write_bg_reset(buf);
                        current_bottom = None;
                    }
                    _ => {}
                }

                let _ = write!(buf, "{ch}");
            }
        }

        self.current_top = current_top;
        self.current_bottom = current_bottom;

        self.swap_buffers();
        self.clear_back_buffer();
    }

    pub fn render(&mut self) -> String {
        // NOTE: estimating 40 bytes worse case for a foreground+background+half-cell output
        let mut buf = String::with_capacity(self.width() * self.height() * 40);
        self.render_to(&mut buf);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_only_outputs_changed_pixels() {
        let mut canvas = HalfCellCanvas::new((6, 1), (0, 0));

        // fill the canvas
        for x in 0..canvas.width() {
            canvas.set_color(x, 0, Color::Rgb(0, 0, 0));
        }

        // render
        let _ = canvas.render();

        // fill canvas again; changing the first and last pixel
        for x in 1..canvas.width() - 1 {
            canvas.set_color(x, 0, Color::Rgb(0, 0, 0));
        }
        canvas.set_color(0, 0, Color::Rgb(100, 100, 100));
        canvas.set_color(canvas.width() - 1, 0, Color::Rgb(200, 200, 200));

        // render again and look for a "move" escape seq
        let output = canvas.render();
        assert!(output.contains(&format!("\x1b[{};{}H", 1, canvas.width())));
    }

    #[test]
    fn cleared_cell_renders_space_not_half_block() {
        let mut canvas = HalfCellCanvas::new((3, 1), (0, 0));

        // frame 1: color one cell (top pixel only)
        canvas.set_color(1, 0, Color::Rgb(255, 0, 0));
        let _ = canvas.render();

        // frame 2: don't set that cell, so it should be cleared
        let output = canvas.render();

        // both halves are None, so it should emit a space (not ▀)
        assert!(
            output.contains(' '),
            "expected space for fully cleared cell, got: {output:?}"
        );
        assert!(
            !output.contains('▀'),
            "should not contain ▀ for fully cleared cell, got: {output:?}"
        );
    }

    #[test]
    fn only_bottom_colored_uses_lower_half_block() {
        let mut canvas = HalfCellCanvas::new((3, 1), (0, 0));

        // color only the bottom pixel of cell at col 1
        canvas.set_color(1, 1, Color::Rgb(0, 255, 0));
        let output = canvas.render();

        assert!(
            output.contains('▄'),
            "expected ▄ when only bottom is colored, got: {output:?}"
        );
    }

    #[test]
    fn set_color_ignores_out_of_bounds_input() {
        let mut canvas = HalfCellCanvas::new((20, 40), (0, 0));

        // shouldn't panic
        canvas.set_color(25, 10, Color::Rgb(0, 0, 0));
        canvas.set_color(10, 90, Color::Rgb(0, 0, 0));
        canvas.set_color(25, 90, Color::Rgb(0, 0, 0));

        // no colors should have been set
        let (_, buf) = canvas.buffers();
        assert!(buf.iter().all(|c| c.is_none()));
    }
}
