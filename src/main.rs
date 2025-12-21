use macroquad::prelude::*;



#[derive(Clone, Copy, Debug)]
struct Shape {
    extent: f32, // generic: distance from center to edge (radius or half-side)
    speed: f32, // pixels per second
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
        draw_rectangle(self.x - self.extent, self.y - self.extent, side, side, color);
    }

    fn collides_with(&self, other: &Shape) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let distance_squared = dx * dx + dy * dy;
        let radius_sum = self.extent + other.extent;
        distance_squared <= radius_sum * radius_sum
    }
}


#[macroquad::main("My game")]
async fn main() {
    rand::srand(miniquad::date::now() as u64);
    const MOVEMENT_SPEED: f32 = 200.0;
    let mut squares: Vec<Shape> = Vec::new();
    let mut game_over = false;
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

        if !game_over{
            clear_background(BLACK);

            // Spawn new square with 5% chance each frame
            if rand::gen_range(0, 99) >= 95 {
                let size = rand::gen_range(10.0, 30.0);
                let extent = size / 2.0;

                squares.push(Shape {
                    extent: extent,
                    speed: rand::gen_range(50.0, 150.0),
                    x : rand::gen_range(extent, screen_width() - extent),
                    y : -extent,
                    collided: false,
                });
            };

            // Build direction from key states:
            // Right -> +1, Left -> -1, both/none -> 0 (same for up/down).
            let dir_x = (is_key_down(KeyCode::Right) as i32 - is_key_down(KeyCode::Left) as i32) as f32;
            let dir_y = (is_key_down(KeyCode::Down) as i32 - is_key_down(KeyCode::Up) as i32) as f32;
            circle.move_by_speed(dir_x, dir_y, dt);


            if is_key_pressed(KeyCode::Space)
            {
                bullets.push(Shape {
                    extent: 5.0,
                    speed: circle.speed * 2.0,
                    x: circle.x,
                    y: circle.y,
                    collided: false,
                });
            }

            // Update squares (they fall down)
            for s in &mut squares {
                s.move_by(0.0, s.speed * dt);
            }


            // Update bullets (they go up)
            for s in &mut bullets {
                s.move_by(0.0, -s.speed * dt);
            }
            // Check bullet-square collisions
            for square in &mut squares {
                for bullet in &mut bullets {
                    if square.collides_with(bullet) {
                        square.collided = true;
                        bullet.collided = true;
                    }
                }
            }
            // Remove off-screen squares
            squares.retain(|s| s.y - s.extent - 1. <= screen_height());
            // Remove off-screen bullets
            bullets.retain(|b| b.y + b.extent + 1. >= 0.0);
            // Check bullet-square collisions
            squares.retain(|square| !square.collided);
            bullets.retain(|bullet| !bullet.collided);


            // Check collisions
            for s in &squares {
                if circle.collides_with(s) {
                    game_over = true;
                }
            }
            // Draw
            circle.draw_circle(YELLOW);
            for s in &squares {
                s.draw_square(RED);
            }
            for b in &mut bullets {
                b.draw_circle(GREEN);
            }


        }
        next_frame().await;
        if game_over && is_key_down(KeyCode::Space) {
            // Reset game state
            squares.clear();
            bullets.clear();
            circle.x = screen_width() / 2.0;
            circle.y = screen_height() / 2.0;
            game_over = false;
        }
    }

}

