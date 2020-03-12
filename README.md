# snakes
a snake game created by rust lang

###### 为什么写这篇日志
或许是rust较为小众，或许是配套的库常常进行不向下兼容的更新，导致很多github上的例程很难直接运行。更难过的是，中文网页中rust相关的学习资料相较于另外一些常见的语言，简直是少太多了。以贪吃蛇这种级别的程序为例，我看到之前有两篇相关的构建教程都相继太监了，而其他的视频，无非只是当年从github上下载下来运行的结果而已，并不知道各个模块都在干什么。因此，一方面是为了自我的学习，另一方面也是为之后可能会有需求的同志们提供一些力所能及的帮助，我将利用rust与piston游戏引擎实现的贪吃蛇整理在这篇文档中，具体将会涉及基于rust语言开发的piston引擎的基本架构，贪吃蛇各种功能的rust实现以及在图像中显示文字等等操作。

说明：实现过程中参考了github上相关snake game的代码，其中主要参考的那个ID叫什么我竟然找不到了，真的是非常抱歉，从他那里学习了很多，默默对他表示感谢。

###### 先看效果
代码下载：[github](https://github.com/obotisr/snakes)
![在这里插入图片描述](https://img-blog.csdnimg.cn/20200312181509906.JPG?x-oss-process=image/watermark,type_ZmFuZ3poZW5naGVpdGk,shadow_10,text_aHR0cHM6Ly9ibG9nLmNzZG4ubmV0L3Bpa2FsaXU=,size_16,color_FFFFFF,t_70)

###### 编译各种依赖库时网络太慢怎么办？
本文使用的rust版本为：
```bash
stable-x86_64-pc-windows-msvc (default)
rustc 1.41.1 (f3e1a954d 2020-02-24)
```

感谢中科大镜像，不管是当年的linux还是现在的rust。总之呢，就是中科大把crates.io这个库给备份了一遍，而且每两小时更新一次，厉害了。具体的修改方法见下面连接内容。
```bash
https://lug.ustc.edu.cn/wiki/mirrors/help/rust-crates
```

###### 依赖库以及toml文件
在toml文件中输入依赖库。其中前四个是piston给出的example直接得到的，主要是构造窗口以及窗口渲染绘图。最后一个是随机数依赖库，在rust-lang book的a guessing game这一章节有提到。
```cpp
[dependencies]
piston = "0.49.0"
piston2d-graphics = "0.36.0"
pistoncore-glutin_window = "0.63.0"
piston2d-opengl_graphics = "0.72.0"
rand = "0.7"
```
###### 二进制文件头引用
搞完toml文件，我们就可以开始编写main.c文件了，这里特别说明一下，整个程序只需要一个main.c文件就可以了，并且将其放在src文件夹之下即可。引入需要的模块，具体如下：
```cpp
use glutin_window::GlutinWindow as Window;		// 构建窗口
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};  // 图形文字渲染
use piston::event_loop::{EventLoop, EventSettings, Events};  // 引擎迭代器
use piston::input::*;	// 外部触发输入，例如键盘鼠标等
use piston::window::WindowSettings;  // 构建窗口
use rand::Rng;	// 随机数
```

###### main函数主体
main.c里面主要包含三部分内容，分别是1.main函数主体，2.app结构及其方法，3.剩余枚举类型。废话少说，首先上第一部分main函数的代码，对应的解释直接写在注释里好了。

```cpp
fn main() {
    // piston标准结构，与渲染有关
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("snakes", [640, 480])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e));

    // 这里App是主要的结构体，构建一个理论尺寸为(80, 60)的结构体，具体的实现见下节
    let mut app = App::new(80, 60);

    // piston标准结构，与渲染有关
    let mut gl = GlGraphics::new(opengl);

    // 这个地方是与老版本不一样的地方，为了能够渲染文字，需要实现读取字体缓存
    // 特别地，老版本不需要TextureSettings的配套
    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let mut glyph_cache = GlyphCache::new("assets/Roboto-Regular.ttf", (), texture_settings)
        .expect("Error unwrapping fonts");
        
    // Create a new event and set the number of updates per second.
    let ref mut events = Events::new(EventSettings::new());
    events.set_ups(60);

    println!("Start loop!");
    // piston 引擎的主要循环，是以迭代器的形式实现的。
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
```

需要特别说明的是，关于字体这部分，因为版本更新导致函数的使用方法都特么变了，我找了很多地方，最后在下面这个链接中找到了具体的使用方法。

```bash
https://github.com/PistonDevelopers/Piston-Tutorials/blob/master/roguelike/src/main.rs
```

###### App主体结构及其方法
除了上面的main.c文件，第二部分就是App结构的定义和方法了，需要说明的是，rust语言并没有所谓cpp中class的概念，但是仍然可以实现“面向对象编程”。另外，因为这部分代码量较大，这里只给出主要结构，具体的代码可以看最开头的github地址。
```cpp
// App结构体
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

// App方法实现
impl App {
	// 新建方法
    pub fn new(horizontal_block_num: u32, vertical_block_num: u32) -> Self {
            // 具体内容省略 //
            // 主要根据输入的窗口理论尺寸并结合基本逻辑，实现App结构体初始化//
    }
    // 渲染方法更新主函数
    fn render(&mut self, args: &RenderArgs, gl: &mut GlGraphics, glyph_cache: &mut GlyphCache) {
        // 具体内容省略 //
        // 包含游戏结束的渲染、游戏正常运行的渲染（蛇头部方块、身体方块、食物方块、场地边缘、游戏分数）
    }
          
    // 结束画面渲染方法
    fn gameover_render(
        &self,
        args: &RenderArgs,
        gl: &mut GlGraphics,
        glyph_cache: &mut GlyphCache,
    ) {
	    // 具体内容省略 //
        // 包含游戏结束信息与分数的渲染显示
    }

	// 渲染过程中Block的尺度变换
    fn renderable_rect(&self, block: &Block, args: &RenderArgs) -> [f64; 4] {
        // 具体内容省略 //
        // 将理论尺寸转化到屏幕尺寸的计算
    }

	// 理论计算更新主函数
    fn update(&mut self, args: &UpdateArgs) {
    	// 具体内容省略 //
        // 包含游戏重新开始的相关操作，        

        // Accumulate time to next update
        self.update_time += args.dt;

        // We update the game logic in fixed intervalls
        if self.update_time >= (1.0 / self.velocity) {
            // 具体内容省略 //
            // 运动方向的锁定（防止蛇180度大掉头，哈哈），碰撞检测，游戏结束判定，蛇头坐标更新，蛇身前移，蛇身加长

            // Initial time
            self.update_time = 0.0;
        }
    }

	// 蛇身加长操作方法
    pub fn growth_action(&mut self) {
        // 具体内容省略 //
        // 立起蛇身加长flag，增加速度，新建食物，增加得分       
    }

	// 碰撞判断
    fn is_collision(&mut self) -> Collision {
        // 具体内容省略 //
        // is the snakehead colliding with the body?
        // is the snakehead colliding with the border?
        // is the snakehead colliding with the fruit?
    }

	// 按键判定，这部分比较独立，全部给出如下，结构其实很清晰
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
```
###### 其他枚举类及头文件
好的，现在来到最后一部分，也就是第三部分关于其他一些杂项。这里都是一些比较基本的东西，包括给颜色单独搞了一个模块，以及Block的结构及其配套的随机数实现方法，还有就是方向与碰撞类型的枚举。需要特别说明的是，如果你自己构建的结构体，要实现包括Clone、比较以及println!输出，那么需要在前面加上#[derive(Debug, Clone, PartialEq)]这样的trait派生，这样就能实现你的目的了。不然，你构建的结构体是没有办法做很多基本操作的。

```cpp
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
```

###### 结尾
以上的内容可以配合bilbili视频以及github一起了解，实际上，如果版本正确的话，应该可以直接运行..的吧。有任何问题，可以留言，谢谢观看。


