use std::{io::Write, time::Duration};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use tixel::{Color, HalfCellCanvas};

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();

    let (cols, rows) = terminal::size()?;
    let mut canvas = HalfCellCanvas::new((rows as usize, cols as usize));

    let height = canvas.height();
    let width = canvas.width();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let mut frame: u64 = 0;
    let mut running = true;

    loop {
        if event::poll(Duration::from_millis(1))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => break,
                KeyCode::Char(' ') => running = !running,
                _ => {}
            }
        }

        if running {
            frame += 1;
            for y in 0..height {
                for x in 0..width {
                    let r = (1. + (frame as f64 / 100.).cos()) / 2. * 256.;
                    let g = (x as f64 / width as f64) * 256.;
                    let b = (1. + (y as f64 / 2.).sin()) / 2. * 256.;
                    canvas.set_color(x, y, Color::new(r as u8, g as u8, b as u8));
                }
            }
        }

        let output = canvas.render();
        let _ = stdout.write(output.as_bytes());
    }

    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
