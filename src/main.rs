use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    ExperimentA,
}

/// Simple frame-based animation from a spritesheet.
struct SpriteAnim {
    texture: Texture2D,
    frame_rects: Vec<Rect>,
    frame_durations: Vec<f32>, // seconds per frame
    t: f32,                    // accumulated time
    playing: bool,
}
impl SpriteAnim {
    fn new(texture: Texture2D, frame_rects: Vec<Rect>, frame_durations: Vec<f32>) -> Self {
        assert!(
            frame_rects.len() == frame_durations.len() && !frame_rects.is_empty(),
            "frame_rects and frame_durations must be same non-zero length"
        );

        Self {
            texture,
            frame_rects,
            frame_durations,
            t: 0.0,
            playing: false,
        }
    }

    fn start(&mut self) {
        self.playing = true;
        self.t = 0.0;
    }

    fn stop(&mut self) {
        self.playing = false;
        self.t = 0.0;
    }

    fn toggle_pause(&mut self) {
        self.playing = !self.playing;
    }

    fn update(&mut self, dt: f32) {
        if self.playing {
            self.t += dt;
        }
    }

    /// Returns current frame index, looping forever when playing.
    fn current_frame(&self) -> usize {
        if !self.playing {
            return 0;
        }

        let total: f32 = self.frame_durations.iter().sum();
        if total <= 0.0 {
            return 0;
        }

        // Loop time into [0, total)
        let mut time = self.t % total;

        for (i, d) in self.frame_durations.iter().enumerate() {
            if time < *d {
                return i;
            }
            time -= *d;
        }

        0
    }

    fn draw(&self, pos: Vec2, scale: f32) {
        let i = self.current_frame();
        let src = self.frame_rects[i];

        draw_texture_ex(
            &self.texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(src.w * scale, src.h * scale)),
                ..Default::default()
            },
        );
    }
}

#[macroquad::main("My game")]
async fn main() {
    let mut game_state: GameState = GameState::MainMenu;

    // Load the spritesheet .
    let texture: Texture2D = load_texture("assets/ghost.png")
        .await
        .expect("Failed to load assets/ghost.png");

    // Pixel-art: avoid blurry scaling.
    texture.set_filter(FilterMode::Nearest);

    // 5 frames, each 32x32, x offsets: 0, 32, 64, 96, 128 (all y=0).
    let frame_rects = vec![
        Rect::new(0.0, 0.0, 32.0, 32.0),
        Rect::new(32.0, 0.0, 32.0, 32.0),
        Rect::new(64.0, 0.0, 32.0, 32.0),
        Rect::new(96.0, 0.0, 32.0, 32.0),
        Rect::new(128.0, 0.0, 32.0, 32.0),
    ];
    // duration: 100ms each -> 0.1s each
    let frame_durations = vec![0.1; 5];

    let mut anim = SpriteAnim::new(texture, frame_rects, frame_durations);

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                if is_key_pressed(KeyCode::A) {
                    game_state = GameState::ExperimentA;
                    anim.stop(); // reset each time you enter
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }

                draw_text(
                    "Press A to visualize the animation. Escape to quit.",
                    screen_width() / 2.0 - 400.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );
            }
            GameState::ExperimentA => {
                draw_text(
                    "ENTER: start  |  SPACE: pause/resume  |  R: reset  |  ESC: back",
                    20.0,
                    40.0,
                    26.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::MainMenu;
                }
                if is_key_pressed(KeyCode::Enter) {
                    anim.start();
                }
                if is_key_pressed(KeyCode::Space) {
                    anim.toggle_pause();
                }
                if is_key_pressed(KeyCode::R) {
                    anim.start();
                }

                // Update + draw
                anim.update(dt);

                let scale = 6.0;
                let sprite_w = 32.0 * scale;
                let sprite_h = 32.0 * scale;

                let pos = vec2(
                    screen_width() * 0.5 - sprite_w * 0.5,
                    screen_height() * 0.5 - sprite_h * 0.5,
                );

                anim.draw(pos, scale);
            }
        }

        next_frame().await;
    }
}
