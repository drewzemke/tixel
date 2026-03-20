use std::{
    collections::VecDeque,
    io::Write,
    thread,
    time::{Duration, Instant},
};

use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::RngExt;
use tixel::{Color, HalfCellCanvas};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl Dir {
    pub fn turn(&self, dir: Self) -> Self {
        match (self, dir) {
            (Dir::Up, Dir::Left) => Dir::Left,
            (Dir::Up, Dir::Right) => Dir::Right,
            (Dir::Down, Dir::Left) => Dir::Left,
            (Dir::Down, Dir::Right) => Dir::Right,
            (Dir::Left, Dir::Up) => Dir::Up,
            (Dir::Left, Dir::Down) => Dir::Down,
            (Dir::Right, Dir::Up) => Dir::Up,
            (Dir::Right, Dir::Down) => Dir::Down,
            (s, _) => *s,
        }
    }
}

struct SnakeGame {
    width: i64,
    height: i64,
    food: (i64, i64),
    snake: VecDeque<(i64, i64)>,
    dir: Dir,
}

impl SnakeGame {
    fn new(width: i64, height: i64) -> Self {
        let snake = vec![(width / 2, height / 2)].into();
        Self {
            width,
            height,
            food: Self::gen_food_loc(width, height, &snake),
            snake,
            dir: Dir::Right,
        }
    }

    fn gen_food_loc(width: i64, height: i64, exclude: &VecDeque<(i64, i64)>) -> (i64, i64) {
        let mut pt = (
            rand::rng().random_range(0..width),
            rand::rng().random_range(0..height),
        );

        while exclude.contains(&pt) {
            pt = (
                rand::rng().random_range(0..width),
                rand::rng().random_range(0..height),
            );
        }

        pt
    }

    fn reset(&mut self) {
        *self = Self::new(self.width, self.height);
    }

    fn head_pos(&self) -> (i64, i64) {
        self.snake[0]
    }

    fn advance(&mut self) {
        let mut new_head = self.head_pos();
        match self.dir {
            Dir::Up => new_head.1 -= 1,
            Dir::Down => new_head.1 += 1,
            Dir::Left => new_head.0 -= 1,
            Dir::Right => new_head.0 += 1,
        }

        let last = self.snake.pop_back();
        self.snake.push_front(new_head);

        // eat?
        if self.head_pos() == self.food {
            self.food = Self::gen_food_loc(self.width, self.height, &self.snake);
            if let Some(last) = last {
                self.snake.push_back(last);
            }
        }
    }

    fn turn(&mut self, dir: Dir) {
        self.dir = self.dir.turn(dir);
    }

    fn is_dead(&self) -> bool {
        self.head_pos().0 < 0
            || self.head_pos().0 >= self.width
            || self.head_pos().1 < 0
            || self.head_pos().1 >= self.height
            || self.snake.iter().skip(1).any(|&x| x == self.head_pos())
    }
}

const TARGET_FRAME_TIME: Duration = Duration::from_millis(50);

fn main() -> anyhow::Result<()> {
    let mut stdout = std::io::stdout();

    let (cols, rows) = terminal::size()?;
    let mut canvas = HalfCellCanvas::new(rows as usize, cols as usize);

    let height = canvas.height();
    let width = canvas.width();

    terminal::enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen, Hide, Clear(ClearType::All))?;

    let mut running = true;

    let mut board = SnakeGame::new(width as i64, height as i64);

    loop {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(10))?
            && let Event::Key(KeyEvent { code, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => break,
                KeyCode::Char(' ') => running = !running,
                KeyCode::Up => board.turn(Dir::Up),
                KeyCode::Down => board.turn(Dir::Down),
                KeyCode::Left => board.turn(Dir::Left),
                KeyCode::Right => board.turn(Dir::Right),
                KeyCode::Char('r') => {
                    board.reset();
                    running = true;
                }
                _ => {}
            }
        }

        if running {
            if board.is_dead() {
                running = false;
            } else {
                // board
                for y in 0..height {
                    for x in 0..width {
                        canvas.set_color(x, y, Color::new(10, 10, 10));
                    }
                }

                // snake
                for pos in &board.snake {
                    // FIXME: doesn't work
                    let color = if board.is_dead() {
                        Color::new(100, 100, 100)
                    } else {
                        Color::new(140, 240, 140)
                    };
                    canvas.set_color(pos.0 as usize, pos.1 as usize, color);
                }

                // food
                canvas.set_color(
                    board.food.0 as usize,
                    board.food.1 as usize,
                    Color::new(240, 140, 140),
                );

                board.advance();
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < TARGET_FRAME_TIME {
            let remaining = TARGET_FRAME_TIME - elapsed;
            thread::sleep(remaining);
        }

        let output = canvas.render();
        let _ = stdout.write(output.as_bytes());
    }

    execute!(stdout, LeaveAlternateScreen, Show)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
