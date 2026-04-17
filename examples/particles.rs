use std::{
    f64::consts::PI,
    io::Write,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::RngExt;
use tixel::{BrailleCanvas, Color};

struct Particle {
    pos: (f64, f64),
    vel: (f64, f64),
    color: Color,
}

impl Particle {
    fn new(pos: (f64, f64), vel: (f64, f64), color: Color) -> Self {
        Self { pos, vel, color }
    }

    fn update(&mut self, dt_sec: f64, width: f64, height: f64) {
        self.pos.0 += self.vel.0 * dt_sec;
        self.pos.1 += self.vel.1 * dt_sec;

        if self.pos.0 <= 0.0 {
            self.pos.0 = 0.1;
            self.vel.0 = -self.vel.0;
        }
        if self.pos.1 <= 0.0 {
            self.pos.1 = 0.1;
            self.vel.1 = -self.vel.1;
        }
        if self.pos.0 >= width {
            self.pos.0 = width - 0.1;
            self.vel.0 = -self.vel.0;
        }
        if self.pos.1 >= height {
            self.pos.1 = height - 0.1;
            self.vel.1 = -self.vel.1;
        }
    }
}

const POLL_TIME: Duration = Duration::from_millis(10);
const NUM_PARTICLES: usize = 500;
const VEL_RANGE: std::ops::Range<f64> = 10.0..15.0;

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();

    let (cols, rows) = terminal::size()?;
    let mut canvas = BrailleCanvas::new(
        (3 * rows as usize / 4, 3 * cols as usize / 4),
        (rows as usize / 8, cols as usize / 8),
    );

    canvas.set_bg_color((0, 0, 0).into());

    let height = canvas.height();
    let width = canvas.width();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let mut running = true;
    let mut rng = rand::rng();

    let mut particles: Vec<Particle> = (0..NUM_PARTICLES)
        .map(|_| {
            let x = rng.random_range(0.0..width as f64);
            let y = rng.random_range(0.0..height as f64);
            let th = rng.random_range(0.0..2. * PI);
            let r = rng.random_range(VEL_RANGE);
            let vx = r * th.cos();
            let vy = r * th.sin();

            let color: (u8, u8, u8) = rng.random();
            Particle::new((x, y), (vx, vy), color.into())
        })
        .collect();

    let mut time = Instant::now();

    loop {
        if event::poll(POLL_TIME)?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => break,
                KeyCode::Char(' ') => running = !running,
                _ => {}
            }
        }

        if running {
            let dt = time.elapsed().as_secs_f64();
            time = std::time::Instant::now();

            for p in &mut particles {
                p.update(dt, width as f64, height as f64);
                canvas.set_f(p.pos.0, p.pos.1, p.color);
            }

            let output = canvas.render();
            let _ = stdout.write(output.as_bytes());
        }
    }

    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
