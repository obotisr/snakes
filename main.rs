extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::window::WindowSettings;
use piston::event_loop::{Events, EventSettings, EventLoop};
use piston::input::*;
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
    pub const ANGEL: [f32; 4 ] = [0.5,0.5,1.0,0.5];
    pub const GREEN: [f32; 4 ] = [0.0,0.5,0.0,1.0];
}

#[derive(Debug,Clone)]
struct Block{
    pos_x : i32,
    pos_y : i32,
}

impl Block {
    pub fn randnew(horizontal_block_num: u32, vertical_block_num: u32) -> Self{
        Block{   pos_x: rand::thread_rng().gen_range(1, (horizontal_block_num - 1) as i32),
                 pos_y: rand::thread_rng().gen_range(1, (vertical_block_num - 1) as i32)}
    }
}

#[derive(PartialEq)]
enum Direction{
    Up,
    Down,
    Left,
    Right
}

enum Collision{
    WithFruit,
    WithSnake,
    WithBorder,
    NoCollision,
}

pub struct App {
    velocity: f64,  // move speed
    direction: Direction,   // direction
    direction_lock: bool,   // set true if don't want to change the direction
    growth_flag: bool,      // set true if body_block number increase
    head_block: Block,      // position of snake head block
    body_block: Vec<Block>, // positions of snake body blocks
    fruit_block: Block,     // position of the fruit
    circus: [u32; 2],       // moving space
    update_time: f64,       // record the time after one update

}

impl App {
    pub fn new(horizontal_block_num: u32, vertical_block_num: u32) -> Self{
        let center_x = ((horizontal_block_num as f64) * 0.5) as i32;
        let center_y = ((vertical_block_num as f64) * 0.5) as i32;

        App{
            velocity : 4.0,
            circus: [horizontal_block_num, vertical_block_num],
            head_block: Block {pos_x: center_x, pos_y: center_y },
            body_block: vec![Block {pos_x: center_x + 1, pos_y: center_y},
                            Block {pos_x: center_x + 2, pos_y: center_y},
                            Block {pos_x: center_x + 3, pos_y: center_y},
                            Block {pos_x: center_x + 4, pos_y: center_y}],
            fruit_block: Block::randnew(horizontal_block_num, vertical_block_num),
            direction: Direction::Left,
            update_time: 0.0,
            direction_lock: false,
            growth_flag: false,
        }
    }

    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics) {
        use graphics::*;
        // draw viewport
        gl.draw(args.viewport(), |c, gl| {
            // clear the screen
            clear(game_colors::BLACK, gl);

            // draw snakes head
            rectangle(game_colors::RED, self.renderable_rect(&self.head_block, args), c.transform, gl);

            // draw fruit
            rectangle(game_colors::RED, self.renderable_rect(&self.fruit_block, args), c.transform, gl);

            // draw borders of the game，
            let vertical_line_radius = (args.window_size[0] as f64) / (self.circus[0] as f64) * 0.5;
            let horizontal_line_radius = (args.window_size[1] as f64) / (self.circus[1] as f64) * 0.5;
            line(game_colors::GREEN,
                vertical_line_radius,
                [0.0, 0.0, 0.0, args.window_size[1] as f64],
                c.transform,
                gl);
            line(game_colors::GREEN,
                vertical_line_radius,
                [args.window_size[0] as f64, 0.0, args.window_size[0] as f64, args.window_size[1] as f64],
                c.transform,
                gl);
            line(game_colors::GREEN,
                horizontal_line_radius,
                [0.0, 0.0, args.window_size[0] as f64, 0.0],
                c.transform,
                gl);
            line(game_colors::GREEN,
                horizontal_line_radius,
                [0.0, args.window_size[1] as f64, args.window_size[0] as f64, args.window_size[1] as f64],
                c.transform,
                gl);

            // draw snakes body
            for block in self.body_block.iter(){
                rectangle(color::WHITE, self.renderable_rect(block, args), c.transform, gl);
            }
        });
    }

    fn renderable_rect(&self, block: &Block, args: &RenderArgs) -> [f64; 4]{
        use graphics::*;
        let block_size_x = args.window_size[0] / (self.circus[0] as f64);
        let block_size_y = args.window_size[1] / (self.circus[1] as f64);
        let window_pos_x = (block.pos_x as f64) * block_size_x;
        let window_pos_y = (block.pos_y as f64) * block_size_y;
        rectangle::rectangle_by_corners(window_pos_x - block_size_x * 0.5,
                                        window_pos_y - block_size_y * 0.5,
                                        window_pos_x + block_size_x * 0.5,
                                        window_pos_y + block_size_y * 0.5)
    }

    fn update(&mut self, args: &UpdateArgs) {
        // Look for collision
        match self.is_collision(){
            Collision::WithFruit => self.growth_action(),
            Collision::NoCollision => (),
            _ => (), // self.game_over = true;
        }

        // Accumulate time to next update
        self.update_time += args.dt;

        // We update the game logic in fixed intervalls
        if self.update_time >= (1.0 / self.velocity){
            // unlock vary of direction
            self.direction_lock = false;

            // position movement
            let (x,y) = match self.direction{
                Direction::Up =>    (0,-1),
                Direction::Down =>  (0,1),
                Direction::Right => (1,0),
                Direction::Left =>  (-1,0),
            };

            // Clone current headposition, will be part of the body
            let mut pre_block = self.head_block.clone();

            // Update position of snake head
            self.head_block.pos_x = self.head_block.pos_x + x;
            self.head_block.pos_y = self.head_block.pos_y + y;

            // "Move" the snake by pushing current blocks of snakebody to new vector
            let mut blocks = Vec::new();
            for block in self.body_block.iter_mut(){
                blocks.push(pre_block);
                pre_block = block.clone();
            }

            // If growing flag is set, don’t waste any block.
            if self.growth_flag{
                blocks.push(pre_block);
                self.growth_flag = false;
            }

            // Assign new body
            self.body_block = blocks;
            // Initial time
            self.update_time = 0.0;
        }
    }

    pub fn growth_action(&mut self){
        // setup growth_flag
        self.growth_flag = true;
        // increase moving speed
        self.velocity += 0.01;
        // simply pick a random location, might also be on the snake
        self.fruit_block = Block::randnew(self.circus[0], self.circus[1]);
    }

    fn is_collision(&mut self)->Collision{
        // // is the snakehead colliding with the body?
        // for block in self.snake_body.iter(){
        //     if self.snake_head.pos_x == block.pos_x && self.snake_head.pos_y == block.pos_y{
        //         return Collision::WithSnake;
        //     }
        // }
        // // is the snakehead colliding with the border?
        // if self.snake_head.pos_x <= 0 || self.snake_head.pos_x >= self.dimensions[0] as i32
        // || self.snake_head.pos_y <= 0 || self.snake_head.pos_y >= self.dimensions[1] as i32{
        //     return Collision::WithBorder;
        // }

        // is the snakehead colliding with the fruit?
        if self.head_block.pos_x == self.fruit_block.pos_x && self.head_block.pos_y == self.fruit_block.pos_y{
            return Collision::WithFruit;
        }
        Collision::NoCollision
    }


    pub fn press(&mut self, button: &Button){
        match button {
            &Button::Keyboard(key) => self.key_press(key),
            _ => {}
        }
    }

    pub fn key_press(&mut self, key: Key) {
        if self.direction_lock == false {
            self.direction_lock = true;
            match key {
                Key::W if self.direction != Direction::Down => {
                    self.direction = Direction::Up;
                }
                Key::S if self.direction != Direction::Up => {
                    self.direction = Direction::Down;
                }
                Key::A if self.direction != Direction::Right => {
                    self.direction = Direction::Left;
                }
                Key::D if self.direction != Direction::Left => {
                    self.direction = Direction::Right;
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
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) });

    // Create a new game and run it.
    let mut app = App::new(80, 60);

    let mut gl = GlGraphics::new(opengl);

    // Create a new event and set the number of updates per second.
    let mut events = Events::new(EventSettings::new());
    events.set_ups(60);

    println!("Start loop!");
    while let Some(e) = events.next(&mut window) {
            if let Some(args) = e.render_args() {
                app.render(&args, &mut gl);
            }
            if let Some(args) = e.update_args() {
                app.update(&args);
            }
            if let Some(button) = e.press_args() {
                app.press(&button);
            }
        }

}
