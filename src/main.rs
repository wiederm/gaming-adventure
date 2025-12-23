use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, Copy, Debug)]
struct Shape {
    extent: f32, // generic: distance from center to edge (radius or half-side)
    speed: f32,  // pixels per second
    x: f32,
    y: f32,
    collided: bool,
}

impl Shape {
    fn clamp_to_screen(&mut self) {
        self.x = clamp(self.x, self.extent, screen_width() - self.extent);
        self.y = clamp(self.y, self.extent, screen_height() - self.extent);
    }
    /// Move by a raw delta in pixels (dx, dy), then clamp to the screen bounds.
    fn move_by(&mut self, dx: f32, dy: f32) {
        self.x += dx;
        self.y += dy;
        self.clamp_to_screen();
    }

    /// Move by direction (-1/0/1) scaled by speed and dt.
    fn move_by_speed(&mut self, dir_x: f32, dir_y: f32, dt: f32) {
        self.move_by(self.speed * dir_x * dt, self.speed * dir_y * dt);
    }

    fn draw_circle(&self, color: Color) {
        draw_circle(self.x, self.y, self.extent, color);
    }

    fn draw_square(&self, color: Color) {
        let side = self.extent * 2.0;
        draw_rectangle(
            self.x - self.extent,
            self.y - self.extent,
            side,
            side,
            color,
        );
    }

    fn collides_with(&self, other: &Shape) -> bool {
        // Circle-circle collision (works as a rough approximation even for squares)
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let distance_squared = dx * dx + dy * dy;
        let radius_sum = self.extent + other.extent;
        distance_squared <= radius_sum * radius_sum
    }
}

/// Update + draw one playing frame. Returns updated game state.
fn run_game_loop(
    mut game_state: GameState,
    circle: &mut Shape,
    squares: &mut Vec<Shape>,
    bullets: &mut Vec<Shape>,
    dt: f32,
) -> GameState {
    // Allow pausing while playing
    if is_key_pressed(KeyCode::P) {
        return GameState::Paused;
    }

    // Spawn new squares with a rate per second (frame-rate independent).
    // Example: 2 spawns/sec on average.
    let spawn_per_second = 2.0;
    let spawn_prob_this_frame = (spawn_per_second * dt).clamp(0.0, 1.0);
    if rand::gen_range(0., 1.) < spawn_prob_this_frame {
        let size = rand::gen_range(10.0, 30.0);
        let extent = size / 2.0;

        squares.push(Shape {
            extent,
            speed: rand::gen_range(50.0, 150.0),
            x: rand::gen_range(extent, screen_width() - extent),
            y: -extent,
            collided: false,
        });
    };

    // Build direction from key states:
    // Right -> +1, Left -> -1, both/none -> 0 (same for up/down).
    let dir_x = (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32;
    let dir_y = (is_key_down(KeyCode::Down) as i32 - is_key_down(KeyCode::Up) as i32) as f32;
    circle.move_by_speed(dir_x, dir_y, dt);

    // Fire bullet
    if is_key_pressed(KeyCode::Space) {
        bullets.push(Shape {
            extent: 5.0,
            speed: circle.speed * 2.0,
            x: circle.x,
            y: circle.y,
            collided: false,
        });
    }

    // Update squares (they fall down)
    for s in squares.iter_mut() {
        s.x += 0.0;
        s.y += s.speed * dt;
    }

    // Update bullets (they go up)
    for b in bullets.iter_mut() {
        b.y -= b.speed * dt;
    }
    // Bullet-square collisions (mark for removal)
    // Index-based loops avoid borrow-checker issues when mutating both Vecs.
    for i in 0..squares.len() {
        for j in 0..bullets.len() {
            if squares[i].collides_with(&bullets[j]) {
                squares[i].collided = true;
                bullets[j].collided = true;
            }
        }
    }
    // Remove off-screen squares/bullets
    squares.retain(|s| s.y - s.extent <= screen_height() + 1.0);
    bullets.retain(|b| b.y + b.extent >= -1.0);
    // Check bullet-square collisions
    squares.retain(|square| !square.collided);
    bullets.retain(|bullet| !bullet.collided);

    // Check circle-square collisions -> GameOver
    for s in squares.iter() {
        if circle.collides_with(s) {
            game_state = GameState::GameOver;
            break;
        }
    }
    // Draw
    circle.draw_circle(YELLOW);
    for s in squares.iter() {
        s.draw_square(RED);
    }
    for b in bullets.iter() {
        b.draw_circle(GREEN);
    }
    return game_state;
}

#[macroquad::main("My game")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    const MOVEMENT_SPEED: f32 = 200.0;
    let mut squares: Vec<Shape> = Vec::new();
    let mut game_state: GameState = GameState::MainMenu;
    let mut bullets: Vec<Shape> = Vec::new();

    let mut circle = Shape {
        extent: 10.0,
        speed: MOVEMENT_SPEED / 2.0,
        x: screen_width() / 2.0,
        y: screen_height() / 2.0,
        collided: false,
    };

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::Playing;
                    squares.clear();
                    bullets.clear();
                    circle.x = screen_width() / 2.0;
                    circle.y = screen_height() / 2.0;
                    // score = 0 FIXME:
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }

                draw_text(
                    "Press ENTER to Start",
                    screen_width() / 2.0 - 100.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );
            }
            GameState::Playing => {
                game_state = run_game_loop(game_state, &mut circle, &mut squares, &mut bullets, dt);
            }
            GameState::Paused => {
                draw_text(
                    "Game Paused. Press P to Resume",
                    screen_width() / 2.0 - 150.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::P) {
                    game_state = GameState::Playing;
                }
            }
            GameState::GameOver => {
                draw_text(
                    "Game Over! Press ENTER to Restart",
                    screen_width() / 2.0 - 180.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::MainMenu;
                }
            }
        }

        next_frame().await;
    }
}
