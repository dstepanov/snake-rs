extern crate sdl2;
extern crate rand;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Font;
use sdl2::render::TextureQuery;

static SCREEN_WIDTH: u32 = 800;
static SCREEN_HEIGHT: u32 = 600;

static CELL_SIZE: u32 = 15;

static BACKGROUND_COLOR: Color = Color::RGB(245, 40, 5);
static FOOD_COLOR: Color = Color::RGB(0, 100, 200);
static SNAKE_COLOR: Color = Color::RGB(100, 200, 0);
static GAMEOVER_TEXT_COLOR: Color = Color::RGB(255, 255, 50);

#[derive(Clone, Debug, PartialEq)]
enum State {
    PLAY,
    GAMEOVER
}

#[derive(PartialEq)]
enum Direction {
    LEFT,
    RIGHT,
    UP,
    DOWN
}

impl Direction {
    fn opposite(&self) -> Direction {
        match *self {
            Direction::DOWN => Direction::UP,
            Direction::UP => Direction::DOWN,
            Direction::LEFT => Direction::RIGHT,
            Direction::RIGHT => Direction::LEFT
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: u32,
    y: u32
}

impl Point {
    fn new(x: u32, y: u32) -> Point {
        Point {
            x: x,
            y: y
        }
    }
}

struct SnakeGame {
    area: Rect,
    state: State,
    x_cells_max: u32,
    y_cells_max: u32,
    score: u32,
    direction: Direction,
    body: Vec<Point>,
    fps: u32,
    show_fps: bool,
    food: Point,
    food_generated_time: u32
}

impl SnakeGame {
    fn new(width: u32, height: u32, initial_snake_length: u32, time: u32) -> SnakeGame {
        let mut snake = SnakeGame {
            area: Rect::new(0, 0, width, height),
            state: State::PLAY,
            x_cells_max: width / CELL_SIZE,
            y_cells_max: height / CELL_SIZE,
            score: 0,
            direction: Direction::LEFT,
            body: Vec::new(),
            fps: 0,
            show_fps: false,
            food: Point { x: 0, y: 0 },
            food_generated_time: time
        };

        snake.create_snake(initial_snake_length);
        snake.food = snake.create_food();
        snake
    }

    fn create_snake(&mut self, initial_snake_length: u32) {
        let x_center = self.x_cells_max / 2;
        let y_center = self.y_cells_max / 2;
        for i in 0..initial_snake_length {
            self.body.push(Point::new(x_center + i, y_center));
        }
    }

    fn create_food(&self) -> Point {
        'findEmptyCell: loop {
            let food = Point {
                x: rand::random::<u32>() % self.x_cells_max,
                y: rand::random::<u32>() % self.y_cells_max
            };

            for item in &self.body {
                if *item == food {
                    continue 'findEmptyCell;
                }
            }

            return food;
        }
    }

    fn move_snake(&mut self, time: u32) {
        let head = self.body.first().unwrap().clone();
        let mut next = &mut match self.direction {
            Direction::LEFT => Point::new(if head.x == 0 { self.x_cells_max } else { head.x - 1 }, head.y),
            Direction::RIGHT => Point::new(if head.x == self.x_cells_max { 0 } else { head.x + 1 }, head.y),
            Direction::UP => Point::new(head.x, if head.y == 0 { self.y_cells_max } else { head.y - 1 }),
            Direction::DOWN => Point::new(head.x, if head.y == self.y_cells_max { 0 } else { head.y + 1 })
        };

        for item in &self.body {
            if *next == *item {
                self.state = State::GAMEOVER;
                return;
            }
        }

        let last = self.body.last().unwrap().clone();

        for item in self.body.iter_mut() {
            let n = item.clone();
            item.x = next.x;
            item.y = next.y;
            next.x = n.x;
            next.y = n.y;
        }

        if self.food == head {
            self.body.push(last);
            self.score += (1000.0 / (time - self.food_generated_time) as f32 * 100.0) as u32;
            self.food_generated_time = time;
            self.food = self.create_food();
        }
    }

    fn change_direction(&mut self, direction: Direction) {
        if self.direction.opposite() == direction {
            return
        }
        self.direction = direction;
    }
}

struct SDLRenderer<'a, 'b: 'a> {
    renderer: &'a mut Renderer<'b>,
    font: &'a Font<'b>
}

impl<'a, 'b> SDLRenderer<'a, 'b> {

    fn render(&mut self, snake_game: &SnakeGame) {
        self.renderer.set_draw_color(BACKGROUND_COLOR);
        self.renderer.fill_rect(snake_game.area).unwrap();

        self.render_text(snake_game);

        if snake_game.state == State::PLAY {
            for point in &snake_game.body {
                self.renderer.set_draw_color(SNAKE_COLOR);
                self.renderer.fill_rect(SDLRenderer::rect_at(point.x, point.y)).unwrap();
            }

            self.renderer.set_draw_color(FOOD_COLOR);
            self.renderer.fill_rect(SDLRenderer::rect_at(snake_game.food.x, snake_game.food.y)).unwrap();
        }
        self.renderer.present();
    }

    fn render_text(&mut self, snake_game: &SnakeGame) {
        if snake_game.state == State::GAMEOVER {
            self.render_text_at("push 'N' for new game", GAMEOVER_TEXT_COLOR, 140, 180);
            self.render_text_at("GAMEOVER", GAMEOVER_TEXT_COLOR, 140, 140);
            self.render_text_at(format!("your score: {}", snake_game.score).as_ref(), GAMEOVER_TEXT_COLOR, 140, 220);
        } else {
            self.render_text_align_right(format!("SCORE: {}", snake_game.score).as_ref(), Color::RGBA(255, 255, 255, 0), 10, 10, snake_game);
            if snake_game.show_fps {
                self.render_text_at(format!("FPS: {}", snake_game.fps).as_ref(), Color::RGB(255, 255, 255), 10, 10);
            }
        }
    }

    fn render_text_align_right(&mut self, text: &str, color: Color, x: i32, y: i32, snake_game: &SnakeGame) {
        let surface = self.font.render(text).blended(color).unwrap();
        let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        self.renderer.copy(&mut texture, None, Some(Rect::new(snake_game.area.width() as i32 - x - width as i32, y, width, height))).unwrap();
    }

    fn render_text_at(&mut self, text: &str, color: Color, x: i32, y: i32) {
        let surface = self.font.render(text).blended(color).unwrap();
        let mut texture = self.renderer.create_texture_from_surface(&surface).unwrap();
        let TextureQuery { width, height, .. } = texture.query();
        self.renderer.copy(&mut texture, None, Some(Rect::new(x, y, width, height))).unwrap();
    }

    fn rect_at(x: u32, y: u32) -> Rect {
        Rect::new((x * CELL_SIZE) as i32, (y * CELL_SIZE) as i32, CELL_SIZE, CELL_SIZE)
    }
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let mut timer = sdl_context.timer().unwrap();

    let window = video_subsys.window("Snake", SCREEN_WIDTH, SCREEN_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut snake = SnakeGame::new(SCREEN_WIDTH, SCREEN_HEIGHT, 10, timer.ticks());

    let font = ttf_context.load_font("./font.TTF", 30).unwrap();
    let mut sdl_renderer = window.renderer().build().unwrap();

    let mut game_renderer = SDLRenderer {
        renderer: &mut sdl_renderer,
        font: &font
    };

    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut move_time = timer.ticks();
    let mut fps_time = timer.ticks();

    let speed = 50;
    let mut fps = 0;

    'mainLoop: loop {
        if timer.ticks() - move_time >= speed {
            snake.move_snake(timer.ticks());
            move_time = timer.ticks();
        }

        if timer.ticks() - fps_time >= 1000 {
            snake.fps = fps;
            fps_time = timer.ticks();
            fps = 0;
        }

        fps += 1;

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'mainLoop,
                Event::KeyDown { keycode: Some(Keycode::N), .. } =>
                    snake = SnakeGame::new(SCREEN_WIDTH, SCREEN_HEIGHT, 10, timer.ticks()),
                Event::KeyDown { keycode: Some(Keycode::F), .. } => snake.show_fps = true,
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => snake.change_direction(Direction::UP),
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => snake.change_direction(Direction::DOWN),
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => snake.change_direction(Direction::LEFT),
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => snake.change_direction(Direction::RIGHT),
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainLoop,
                _ => {}
            }
        }

        game_renderer.render(&snake);

        //        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

