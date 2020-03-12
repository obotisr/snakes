use glutin_window::GlutinWindow as Window;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventLoop, EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;

/// Contains colors that can be used in the game
pub mod game_colors {
    pub const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
    pub const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
    pub const BLUE: [f32; 4] = [0.0, 0.0, 1.0, 1.0];
    pub const LIGHTBLUE: [f32; 4] = [0.0, 1.0, 1.0, 1.0];
    pub const ORANGE: [f32; 4] = [1.0, 0.5, 0.0, 1.0];
    pub const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
    pub const PINK: [f32; 4] = [1.0, 0.0, 1.0, 1.0];
    pub const ANGEL: [f32; 4] = [0.5, 0.5, 1.0, 0.5];
    pub const GREEN: [f32; 4] = [0.0, 0.5, 0.0, 1.0];
}

#[derive(Debug, Clone, PartialEq)]
struct Block {
    pos_x: i32,
    pos_y: i32,
}

impl Block {
    pub fn randnew(horizontal_block_num: u32, vertical_block_num: u32) -> Self {
        Block {
            pos_x: rand::thread_rng().gen_range(1, (horizontal_block_num - 1) as i32),
            pos_y: rand::thread_rng().gen_range(1, (vertical_block_num - 1) as i32),
        }
    }
}

#[derive(PartialEq)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

enum Collision {
    WithFruit,
    WithSnake,
    WithBorder,
    NoCollision,
}

pub struct App {
    velocity: f64,          // move speed
    direction: Direction,   // direction
    direction_lock: bool,   // set true if don't want to change the direction
    growth_flag: bool,      // set true if body_block number increase
    gameover_flag: bool,    // set true if game over
    restart_flag: bool,     // set true if game restart
    head_block: Block,      // position of snake head block
    body_block: Vec<Block>, // positions of snake body blocks
    fruit_block: Block,     // position of the fruit
    circus: [u32; 2],       // moving space
    update_time: f64,       // record the time after one update
    score: u32,             // eat a fruit, increase one point
}

impl App {
    pub fn new(horizontal_block_num: u32, vertical_block_num: u32) -> Self {
        let center_x = ((horizontal_block_num as f64) * 0.5) as i32;
        let center_y = ((vertical_block_num as f64) * 0.5) as i32;

        App {
            velocity: 6.0,
            circus: [horizontal_block_num, vertical_block_num],
            head_block: Block {
                pos_x: center_x,
                pos_y: center_y,
            },
            body_block: vec![
                Block {
                    pos_x: center_x + 1,
                    pos_y: center_y,
                },
                Block {
                    pos_x: center_x + 2,
                    pos_y: center_y,
                },
                Block {
                    pos_x: center_x + 3,
                    pos_y: center_y,
                },
                Block {
                    pos_x: center_x + 4,
                    pos_y: center_y,
                },
            ],
            fruit_block: Block::randnew(horizontal_block_num, vertical_block_num),
            direction: Direction::Left,
            update_time: 0.0,
            direction_lock: false,
            growth_flag: false,
            gameover_flag: false,
            restart_flag: false,
            score: 0,
        }
    }

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache) {
        use graphics::*;

        // Only draw the "game over" screen
        if self.gameover_flag {
            self.gameover_render(args, gl, glyph_cache); //, glyph_cache
            return;
        }

        // draw viewport
        gl.draw(args.viewport(), |c, gl| {
            // clear the screen
            clear(game_colors::BLACK, gl);

            // draw snakes head
            rectangle(
                game_colors::RED,
                self.renderable_rect(&self.head_block, args),
                c.transform,
                gl,
            );

            // draw fruit
            rectangle(
                game_colors::RED,
                self.renderable_rect(&self.fruit_block, args),
                c.transform,
                gl,
            );

            // draw borders of the game，
            let vertical_line_radius = (args.window_size[0] as f64) / (self.circus[0] as f64) * 0.5;
            let horizontal_line_radius =
                (args.window_size[1] as f64) / (self.circus[1] as f64) * 0.5;
            line(
                game_colors::LIGHTBLUE,
                vertical_line_radius,
                [0.0, 0.0, 0.0, args.window_size[1] as f64],
                c.transform,
                gl,
            );
            line(
                game_colors::LIGHTBLUE,
                vertical_line_radius,
                [
                    args.window_size[0] as f64,
                    0.0,
                    args.window_size[0] as f64,
                    args.window_size[1] as f64,
                ],
                c.transform,
                gl,
            );
            line(
                game_colors::LIGHTBLUE,
                horizontal_line_radius,
                [0.0, 0.0, args.window_size[0] as f64, 0.0],
                c.transform,
                gl,
            );
            line(
                game_colors::LIGHTBLUE,
                horizontal_line_radius,
                [
                    0.0,
                    args.window_size[1] as f64,
                    args.window_size[0] as f64,
                    args.window_size[1] as f64,
                ],
                c.transform,
                gl,
            );

            // draw snakes body
            for block in self.body_block.iter() {
                rectangle(
                    color::WHITE,
                    self.renderable_rect(block, args),
                    c.transform,
                    gl,
                );
            }

            // draw score
            text(
                color::WHITE,
                15,
                format!("Your score is {}", self.score).as_str(),
                glyph_cache,
                c.transform.trans(10.0, 20.0),
                gl,
            )
            .unwrap();
        });
    }

    fn gameover_render(
        &self,
        args: &RenderArgs,
        gl: &mut GlGraphics,
        glyph_cache: &mut GlyphCache,
    ) {
        use graphics::*;

        // draw viewport
        gl.draw(args.viewport(), |c, gl| {
            // clear the screen
            clear(game_colors::BLACK, gl);
            // display Game over and score
            text(
                color::WHITE,
                15,
                format!("Game over! Press Space to restart, Escape to quit!").as_str(),
                glyph_cache,
                c.transform.trans(10.0, 40.0),
                gl,
            )
            .unwrap();

            text(
                color::WHITE,
                15,
                format!("Your score is {}", self.score).as_str(),
                glyph_cache,
                c.transform.trans(10.0, 20.0),
                gl,
            )
            .unwrap();
            return;
        });
    }

    fn renderable_rect(&self, block: &Block, args: &RenderArgs) -> [f64; 4] {
        use graphics::*;
        let block_size_x = args.window_size[0] / (self.circus[0] as f64);
        let block_size_y = args.window_size[1] / (self.circus[1] as f64);
        let window_pos_x = (block.pos_x as f64) * block_size_x;
        let window_pos_y = (block.pos_y as f64) * block_size_y;
        rectangle::rectangle_by_corners(
            window_pos_x - block_size_x * 0.5,
            window_pos_y - block_size_y * 0.5,
            window_pos_x + block_size_x * 0.5,
            window_pos_y + block_size_y * 0.5,
        )
    }

    fn update(&mut self, args: &UpdateArgs) {
        // restart
        if self.restart_flag {
            let pristine = App::new(self.circus[0], self.circus[1]);
            self.velocity = pristine.velocity;
            self.direction = pristine.direction;
            self.direction_lock = pristine.direction_lock;
            self.growth_flag = pristine.growth_flag;
            self.gameover_flag = pristine.gameover_flag;
            self.restart_flag = pristine.restart_flag;
            self.update_time = pristine.update_time;
            self.fruit_block = pristine.fruit_block;
            self.head_block = pristine.head_block;
            self.body_block = pristine.body_block.clone();
            self.update_time = pristine.update_time;
            self.score = pristine.score;
            return;
        }

        // Accumulate time to next update
        self.update_time += args.dt;

        // We update the game logic in fixed intervalls
        if self.update_time >= (1.0 / self.velocity) {
            // unlock vary of direction
            self.direction_lock = false;

            // Look for collision
            match self.is_collision() {
                Collision::WithFruit => self.growth_action(),
                Collision::NoCollision => (),
                Collision::WithBorder => self.gameover_flag = true,
                Collision::WithSnake => self.gameover_flag = true,
            }

            //
            if self.gameover_flag == true {
                return;
            }

            // position movement
            let (x, y) = match self.direction {
                Direction::Up => (0, -1),
                Direction::Down => (0, 1),
                Direction::Right => (1, 0),
                Direction::Left => (-1, 0),
            };

            // Clone current headposition, will be part of the body
            let mut pre_block = self.head_block.clone();

            // Update position of snake head
            self.head_block.pos_x = self.head_block.pos_x + x;
            self.head_block.pos_y = self.head_block.pos_y + y;

            // "Move" the snake by pushing current blocks of snakebody to new vector
            let mut blocks = Vec::new();
            for block in self.body_block.iter_mut() {
                blocks.push(pre_block);
                pre_block = block.clone();
            }

            // If growing flag is set, don’t waste any block.
            if self.growth_flag {
                blocks.push(pre_block);
                self.growth_flag = false;
            }

            // Assign new body
            self.body_block = blocks;
            // Initial time
            self.update_time = 0.0;
        }
    }

    pub fn growth_action(&mut self) {
        // setup growth_flag
        self.growth_flag = true;
        // increase moving speed
        self.velocity += 0.01;
        // simply pick a random location, might also be on the snake
        self.fruit_block = Block::randnew(self.circus[0], self.circus[1]);
        // increase one point
        self.score += 1;
    }

    fn is_collision(&mut self) -> Collision {
        // is the snakehead colliding with the body?
        for block in self.body_block.iter() {
            if self.head_block == *block {
                return Collision::WithSnake;
            }
        }
        // is the snakehead colliding with the border?
        if self.head_block.pos_x <= 0
            || self.head_block.pos_x >= self.circus[0] as i32
            || self.head_block.pos_y <= 0
            || self.head_block.pos_y >= self.circus[1] as i32
        {
            return Collision::WithBorder;
        }

        // is the snakehead colliding with the fruit?
        if self.head_block == self.fruit_block {
            return Collision::WithFruit;
        }
        Collision::NoCollision
    }

    pub fn press(&mut self, button: &Button) {
        match button {
            &Button::Keyboard(key) => self.key_press(key),
            _ => {}
        }
    }

    pub fn key_press(&mut self, key: Key) {
        if self.direction_lock == false {
            self.direction_lock = true;
            match key {
                Key::Up if self.direction != Direction::Down => {
                    self.direction = Direction::Up;
                }
                Key::Down if self.direction != Direction::Up => {
                    self.direction = Direction::Down;
                }
                Key::Left if self.direction != Direction::Right => {
                    self.direction = Direction::Left;
                }
                Key::Right if self.direction != Direction::Left => {
                    self.direction = Direction::Right;
                }
                Key::Space => {
                    self.restart_flag = true;
                    return;
                }
                _ => {}
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("snakes", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    // Create a new game and run it.
    let mut app = App::new(80, 60);

    let mut gl = GlGraphics::new(opengl);

    // typeface
    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let mut glyph_cache = GlyphCache::new("assets/Roboto-Regular.ttf", (), texture_settings)
        .expect("Error unwrapping fonts");
    // Create a new event and set the number of updates per second.
    let ref mut events = Events::new(EventSettings::new());
    events.set_ups(60);

    println!("Start loop!");
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &mut gl, &mut glyph_cache);
        }
        if let Some(args) = e.update_args() {
            app.update(&args);
        }
        if let Some(button) = e.press_args() {
            app.press(&button);
        }
    }
}
