use std::fmt::Write;

use crate::color::Color;

/// writes a move-to escape seq to a string buffer. NOTE: row and col are *ZERO*-based
pub fn write_move_to(str: &mut String, row: usize, col: usize) {
    let _ = write!(str, "\x1b[{};{}H", row + 1, col + 1);
}

pub fn write_fg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[38;2;{};{};{}m", color.r(), color.g(), color.b());
}

pub fn write_bg_color(str: &mut String, color: Color) {
    let _ = write!(str, "\x1b[48;2;{};{};{}m", color.r(), color.g(), color.b());
}

pub fn write_fg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[39m",);
}

pub fn write_bg_reset(str: &mut String) {
    let _ = write!(str, "\x1b[49m",);
}
