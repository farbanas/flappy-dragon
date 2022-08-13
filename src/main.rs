#![warn(clippy::pedantic)]

use bracket_lib::prelude::{
    self, BError, BTermBuilder, Degrees, PointF, RandomNumberGenerator, NAVY, WHITE,
};

const FONT_PATH: &str = "../resources/flappy32.png";
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 40;
const FRAME_DURATION: f32 = 40.0;
const DRAGON_FRAMES: [u16; 6] = [64, 1, 2, 3, 2, 1];

enum GameMode {
    Menu,
    Playing,
    End,
}

struct Player {
    x: f32,
    y: f32,
    velocity: f32,
    frame: usize,
}

impl Player {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            velocity: 0.0,
            frame: 0,
        }
    }

    fn render(&mut self, ctx: &mut prelude::BTerm) {
        ctx.set_active_console(1);
        ctx.cls();
        ctx.set_fancy(
            PointF::new(0.0, self.y),
            1,
            Degrees::new(0.0),
            PointF::new(2.0, 2.0),
            prelude::WHITE,
            prelude::NAVY,
            DRAGON_FRAMES[self.frame],
        );
    }

    fn gravity_and_move(&mut self) {
        if self.velocity < 2.0 {
            self.velocity += 0.2;
        }

        self.y += self.velocity;
        self.x += 1.0;
        if self.y < 0.0 {
            self.y = 0.0;
        }

        self.x += 1.0;
        self.frame += 1;
        self.frame = self.frame % 6;
    }

    fn flap(&mut self) {
        self.velocity = -2.0;
    }
}

struct Obstacle {
    x: f32,
    gap_y: f32,
    size: f32,
}

impl Obstacle {
    fn new(x: f32, score: i32) -> Self {
        let mut random = RandomNumberGenerator::new();
        Self {
            x,
            gap_y: random.range(10.0, 40.0),
            size: f32::max(2.0, 20.0 - score as f32),
        }
    }

    fn render(&mut self, ctx: &mut prelude::BTerm, player_x: f32) {
        for x in 0..SCREEN_WIDTH {
            ctx.set(x, SCREEN_HEIGHT - 1, WHITE, WHITE, prelude::to_cp437('#'));
            ctx.set_fancy(
                PointF::new(x as f32, SCREEN_HEIGHT as f32 - 1.0),
                1,
                Degrees::new(0.0),
                PointF::new(2.0, 2.0),
                WHITE,
                NAVY,
                prelude::to_cp437('#'),
            );
        }

        let screen_x = self.x - player_x;
        let half_size = self.size / 2.0;
        // Top wall
        for y in 0..(self.gap_y - half_size) as i32 {
            ctx.set_fancy(
                PointF::new(screen_x, y as f32),
                1,
                Degrees::new(0.0),
                PointF::new(2.0, 2.0),
                WHITE,
                NAVY,
                179,
            );
        }

        // Bottom wall - now leaving room for the ground
        for y in (self.gap_y + half_size) as i32..SCREEN_HEIGHT - 1 {
            ctx.set_fancy(
                PointF::new(screen_x, y as f32),
                1,
                Degrees::new(0.0),
                PointF::new(2.0, 2.0),
                WHITE,
                NAVY,
                179,
            );
        }
    }

    fn hit_obstacle(&self, player: &Player) -> bool {
        let half_size = self.size / 2.0;
        player.x == self.x
            && ((player.y) < self.gap_y - half_size || player.y > self.gap_y + half_size)
    }
}

struct State {
    player: Player,
    frame_time: f32,
    obstacle: Obstacle,
    mode: GameMode,
    score: i32,
}

impl State {
    fn new() -> Self {
        Self {
            player: Player::new(5.0, SCREEN_HEIGHT as f32 / 2.0),
            frame_time: 0.0,
            obstacle: Obstacle::new(SCREEN_WIDTH as f32, 0),
            mode: GameMode::Menu,
            score: 0,
        }
    }

    fn main_menu(&mut self, ctx: &mut prelude::BTerm) {
        ctx.cls();
        ctx.print_centered(5, "Welcome to Flappy Dragon");
        ctx.print_centered(8, "(P) Play Game");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                prelude::VirtualKeyCode::P => self.restart(),
                prelude::VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }

    fn play(&mut self, ctx: &mut prelude::BTerm) {
        ctx.cls_bg(NAVY);
        self.frame_time += ctx.frame_time_ms;
        if self.frame_time > FRAME_DURATION {
            self.frame_time = 0.0;

            self.player.gravity_and_move();
        }

        if let Some(prelude::VirtualKeyCode::Space) = ctx.key {
            self.player.flap();
        }

        self.player.render(ctx);

        ctx.print(0, 0, "Press SPACE to flap.");
        ctx.print(0, 1, &format!("Score: {}", self.score));

        self.obstacle.render(ctx, self.player.x);
        if self.player.x > self.obstacle.x {
            self.score += 1;
            self.obstacle = Obstacle::new(self.player.x + SCREEN_WIDTH as f32, self.score);
        }

        if self.player.y as i32 > SCREEN_HEIGHT || self.obstacle.hit_obstacle(&self.player) {
            self.mode = GameMode::End;
        }
    }

    fn restart(&mut self) {
        self.player = Player::new(5.0, SCREEN_HEIGHT as f32 / 2.0);
        self.frame_time = 0.0;
        self.obstacle = Obstacle::new(SCREEN_WIDTH as f32, 0);
        self.mode = GameMode::Playing;
        self.score = 0;
    }

    fn dead(&mut self, ctx: &mut prelude::BTerm) {
        ctx.cls();
        ctx.print_centered(5, "You died!");
        ctx.print_centered(6, &format!("You earned {} points", self.score));
        ctx.print_centered(8, "(P) Play Again");
        ctx.print_centered(9, "(Q) Quit Game");

        if let Some(key) = ctx.key {
            match key {
                prelude::VirtualKeyCode::P => self.restart(),
                prelude::VirtualKeyCode::Q => ctx.quitting = true,
                _ => {}
            }
        }
    }
}

impl prelude::GameState for State {
    fn tick(&mut self, ctx: &mut prelude::BTerm) {
        match self.mode {
            GameMode::Menu => self.main_menu(ctx),
            GameMode::Playing => self.play(ctx),
            GameMode::End => self.dead(ctx),
        }
    }
}

fn main() -> BError {
    let term = BTermBuilder::new()
        .with_font(FONT_PATH, 32, 32)
        .with_simple_console(SCREEN_WIDTH, SCREEN_HEIGHT, FONT_PATH)
        .with_fancy_console(SCREEN_WIDTH, SCREEN_HEIGHT, FONT_PATH)
        .with_title("Flappy Dragon")
        .with_tile_dimensions(16, 16)
        .with_vsync(true)
        .build()?;

    prelude::main_loop(term, State::new())
}
