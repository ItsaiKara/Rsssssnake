extern crate piston;
extern crate graphics;
extern crate glutin_window;
extern crate opengl_graphics;
extern crate rand;

use rand::Rng;
use std::collections::LinkedList;
use std::iter::FromIterator;
use opengl_graphics::TextureSettings;
use piston::window::WindowSettings;
use piston::event_loop::*;
use piston::input::*;
use glutin_window::GlutinWindow;
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache, Filter};

// Embed the font file
const FONT_DATA: &[u8] = include_bytes!("../assets/FiraSans-Regular.ttf");

struct Game {
    gl: GlGraphics,
    snake: Snake,
    apple: Apple,
    glyph_cache: GlyphCache<'static>,
}

#[derive(Clone, Copy, PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

struct Apple {
    x: i32,
    y: i32,
}

impl Apple {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics::*;

        let RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let square = rectangle::square(
            (self.x * 10) as f64,
            (self.y * 10) as f64,
            10_f64
        );

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            rectangle(RED, square, transform, gl);
        });
    }

    fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        self.x = rng.gen_range(0, 30);
        self.y = rng.gen_range(0, 30);
    }
}

struct Snake {
    x: i32,
    y: i32,
    velocity: i32,
    direction: Direction,
    body: LinkedList<(i32, i32)>,
}

impl Snake {
    fn render(&self, gl: &mut GlGraphics, args: &RenderArgs) {
        use graphics::*;

        let GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];

        let squares: Vec<graphics::types::Rectangle> = self.body.iter().map(|&(x, y)| {
            rectangle::square(
                (x * self.velocity) as f64,
                (y * self.velocity) as f64,
                10_f64
            )
        }).collect();

        gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform;
            for square in squares {
                rectangle(GREEN, square, transform, gl);
            }
        });
    }

    fn update(&mut self) {
        let mut next_head = *self.body.front().expect("Snake has no body");

        match self.direction {
            Direction::Up => next_head.1 -= 1,
            Direction::Down => next_head.1 += 1,
            Direction::Left => next_head.0 -= 1,
            Direction::Right => next_head.0 += 1,
        }
        self.body.push_front(next_head);
        self.body.pop_back().unwrap();
    }

    fn grow(&mut self) {
        let mut new_head = *self.body.front().expect("Snake has no body");

        match self.direction {
            Direction::Up => new_head.1 -= 1,
            Direction::Down => new_head.1 += 1,
            Direction::Left => new_head.0 -= 1,
            Direction::Right => new_head.0 += 1,
        }

        self.body.push_front(new_head);
    }

    fn check_collision(game: &mut Game) {
        let (x, y) = game.snake.body.front().unwrap().clone();
        if x < 0 || y < 0 || x >= 30 || y >= 30 {
            panic!("You hit the wall!");
        }

        if game.snake.body.iter().skip(1).any(|&(a, b)| x == a && y == b) {
            panic!("You hit yourself!");
        }

        if x == game.apple.x && y == game.apple.y {
            game.snake.grow();
            game.apple.randomize();
        }
    }

}

impl Game {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        self.gl.draw(args.viewport(), |_c, gl| {
            clear(BLACK, gl);
        });

        self.snake.render(&mut self.gl, args);
        self.apple.render(&mut self.gl, args);

        let pos = format!("({}, {})", self.snake.body.front().unwrap().0, self.snake.body.front().unwrap().1);
        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform.trans(10.0, 20.0);
            Text::new_color([1.0, 1.0, 1.0, 1.0], 12).draw(
                &pos,
                &mut self.glyph_cache,
                &c.draw_state,
                transform, 
                gl
            ).unwrap();
        });

        let arrow = match self.snake.direction {
            Direction::Up => "↑",
            Direction::Down => "↓",
            Direction::Left => "←",
            Direction::Right => "→",
        };

        self.gl.draw(args.viewport(), |c, gl| {
            let transform = c.transform.trans(10., 40.);
            Text::new_color([1.0, 1.0, 1.0, 1.0], 12).draw(
                &arrow,
                &mut self.glyph_cache,
                &c.draw_state,
                transform,
                gl
            ).unwrap();
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        self.snake.update();
    }
}

fn main() {
    let opengl = OpenGL::V3_2;

    let mut rng = rand::thread_rng();

    let mut window: GlutinWindow = WindowSettings::new(
        "Snek",
        [300, 300]
    )
    .opengl(opengl)
    .exit_on_esc(true)
    .build()
    .unwrap();

    // Load the font from the embedded bytes
    let font_data: &[u8] = FONT_DATA;
    let glyph_cache = GlyphCache::from_bytes(font_data, (), TextureSettings::new().filter(Filter::Nearest)).unwrap();

    // Generate apple at random location
    let x_apple: i32 = rng.gen_range(0, 30);
    let y_apple: i32 = rng.gen_range(0, 30);

    let mut game = Game {
        gl: GlGraphics::new(opengl),
        snake: Snake { x: 0, y: 0, velocity: 10, direction: Direction::Right, body: LinkedList::from_iter((vec![(0, 0), (0, 1), (0, 2)]).into_iter()) },
        apple: Apple { x: x_apple, y: y_apple },
        glyph_cache,
    };

    let mut events = Events::new(EventSettings::new()).ups(12);
    while let Some(e) = events.next(&mut window) {
        if let Some(r) = e.render_args() {
            game.render(&r);
        }

        if let Some(u) = e.update_args() {
            game.update(&u);
            Snake::check_collision(&mut game);
        }

        if let Some(Button::Keyboard(key)) = e.press_args() {
            // Prevent snake from going backwards

            let last_direction = game.snake.direction.clone();

            game.snake.direction = match key {
                Key::Up if last_direction != Direction::Down => Direction::Up,
                Key::Down if last_direction != Direction::Up => Direction::Down,
                Key::Left if last_direction != Direction::Right => Direction::Left,
                Key::Right if last_direction != Direction::Left => Direction::Right,
                Key::Space => {
                    game.snake.grow();
                    last_direction
                },
                Key::K => {
                    panic!("You killed the snake!");
                },
                _ => last_direction,
            };
        }
    }
}
