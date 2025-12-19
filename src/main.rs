use macroquad::prelude::*;

#[macroquad::main("My game")]
async fn main() {

    let mut x = screen_width() / 2.0;
    let mut y = screen_height() / 2.0;

    let mut i = 1;

    loop {
        i += 1;
        clear_background(BLACK);
        if is_key_down(KeyCode::Right) {
            x += 1.0;
        }
        if is_key_down(KeyCode::Left) {
            x -= 1.0;
        }
        if is_key_down(KeyCode::Down) {
            y += 1.0;
        }
        if is_key_down(KeyCode::Up) {
            y -= 1.0;
        }

        let r: f32;
        r = 16.0 * (i as f32 / 10.0).sin();

        draw_circle(x, y, r, YELLOW);
        next_frame().await
    }

}
