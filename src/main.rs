use macroquad::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    Demo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MoveState {
    Idle,
    Walk,
    Run,
    Jump,
    Dash,
}

/// Sprite animation helper (unchanged conceptually)
struct SpriteAnim {
    texture: Texture2D,
    frames: Vec<Rect>,
    durations: Vec<f32>,
    t: f32,
}

impl SpriteAnim {
    fn new(texture: Texture2D, frames: Vec<Rect>, durations: Vec<f32>) -> Self {
        Self {
            texture,
            frames,
            durations,
            t: 0.0,
        }
    }

    fn update(&mut self, dt: f32) {
        self.t += dt;
    }

    fn frame_index(&self) -> usize {
        let total: f32 = self.durations.iter().sum();
        let mut time = self.t % total;

        for (i, d) in self.durations.iter().enumerate() {
            if time < *d {
                return i;
            }
            time -= *d;
        }
        0
    }

    fn draw(&self, pos: Vec2, scale: f32, flip_x: bool) {
        let src = self.frames[self.frame_index()];
        draw_texture_ex(
            &self.texture,
            pos.x,
            pos.y,
            WHITE,
            DrawTextureParams {
                source: Some(src),
                dest_size: Some(vec2(src.w * scale, src.h * scale)),
                flip_x,
                ..Default::default()
            },
        );
    }
}

/// Holds one animation per movement state
struct AnimSet {
    idle: SpriteAnim,
    walk: SpriteAnim,
    run: SpriteAnim,
    jump: SpriteAnim,
    dash: SpriteAnim,
}

struct Player {
    pos: Vec2,
    vel: Vec2,
    facing: f32,
    on_ground: bool,
    state: MoveState,
}

fn sheet_frames(frame_count: usize, frame_w: f32, frame_h: f32) -> Vec<Rect> {
    (0..frame_count)
        .map(|i| Rect::new(i as f32 * frame_w, 0.0, frame_w, frame_h))
        .collect()
}

#[macroquad::main("Sprite Demo")]
async fn main() {
    let mut game_state = GameState::MainMenu;

    // === Load sprite sheets ===
    let idle_tex = load_texture("assets/idle1.png")
        .await
        .expect("Failed to load idle1.png");
    let walk_tex = load_texture("assets/idle1.png")
        .await
        .expect("Failed to load walk.png");
    let run_tex = load_texture("assets/idle1.png")
        .await
        .expect("Failed to load run.png");
    let jump_tex = load_texture("assets/idle1.png")
        .await
        .expect("Failed to load jump.png");
    let dash_tex = load_texture("assets/idle1.png")
        .await
        .expect("Failed to load dash.png");
    for t in [&idle_tex, &walk_tex, &run_tex, &jump_tex, &dash_tex] {
        t.set_filter(FilterMode::Nearest);
    }

    let mut anims = AnimSet {
        idle: SpriteAnim::new(idle_tex, sheet_frames(5, 32.0, 32.0), vec![0.2; 5]),
        walk: SpriteAnim::new(walk_tex, sheet_frames(4, 32.0, 32.0), vec![0.12; 4]),
        run: SpriteAnim::new(run_tex, sheet_frames(4, 32.0, 32.0), vec![0.08; 4]),
        jump: SpriteAnim::new(jump_tex, sheet_frames(4, 32.0, 32.0), vec![0.1; 4]),
        dash: SpriteAnim::new(dash_tex, sheet_frames(3, 32.0, 32.0), vec![0.06; 3]),
    };

    let scale = 6.0;
    let ground_y = screen_height() * 0.75 - 32.0 * scale;

    let mut player = Player {
        pos: vec2(screen_width() * 0.5, ground_y),
        vel: vec2(0.0, 0.0),
        facing: 1.0,
        on_ground: true,
        state: MoveState::Idle,
    };

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                draw_text(
                    "Sprite demo\nPress ENTER",
                    screen_width() * 0.5 - 180.0,
                    screen_height() * 0.5,
                    32.0,
                    WHITE,
                );

                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::Demo;
                }
            }

            GameState::Demo => {
                // === Input ===
                if is_key_down(KeyCode::Left) {
                    player.facing = -1.0;
                }
                if is_key_down(KeyCode::Right) {
                    player.facing = 1.0;
                }

                if is_key_pressed(KeyCode::I) {
                    player.state = MoveState::Idle;
                }
                if is_key_pressed(KeyCode::W) {
                    player.state = MoveState::Walk;
                }
                if is_key_pressed(KeyCode::R) {
                    player.state = MoveState::Run;
                }
                if is_key_pressed(KeyCode::J) && player.on_ground {
                    player.state = MoveState::Jump;
                    player.vel.y = -900.0;
                    player.on_ground = false;
                }
                if is_key_pressed(KeyCode::D) {
                    player.state = MoveState::Dash;
                    player.vel.x = player.facing * 1200.0;
                }

                // === Physics ===
                let gravity = 2200.0;

                match player.state {
                    MoveState::Idle => player.vel.x = 0.0,
                    MoveState::Walk => {
                        player.vel.x = player.facing * dt * 10_000.;
                        if player.pos.x > screen_width() {
                            player.pos.x = screen_width() * 0.5;
                        };
                    }
                    MoveState::Run => player.vel.x = player.facing * dt * 10_000. * 2.,
                    MoveState::Jump => player.vel.x = player.facing * dt * 150.,
                    MoveState::Dash => {
                        player.vel.x *= 0.88;
                        if player.vel.x.abs() < 80.0 {
                            player.state = MoveState::Idle;
                        }
                    }
                }

                if !player.on_ground {
                    player.vel.y += gravity * dt;
                }

                player.pos += player.vel * dt;

                if player.pos.y >= ground_y {
                    player.pos.y = ground_y;
                    player.vel.y = 0.0;
                    player.on_ground = true;
                    if player.state == MoveState::Jump {
                        player.state = MoveState::Idle;
                    }
                }

                // === Animation ===
                let anim = match player.state {
                    MoveState::Idle => &mut anims.idle,
                    MoveState::Walk => &mut anims.walk,
                    MoveState::Run => &mut anims.run,
                    MoveState::Jump => &mut anims.jump,
                    MoveState::Dash => &mut anims.dash,
                };

                anim.update(dt);
                anim.draw(player.pos, scale, player.facing < 0.0);

                // === UI ===
                draw_text(
                    "I:Idle W:Walk R:Run J:Jump D:Dash  ← →",
                    20.0,
                    30.0,
                    22.0,
                    WHITE,
                );
                draw_text(
                    &format!("State: {:?}", player.state),
                    20.0,
                    60.0,
                    22.0,
                    YELLOW,
                );
            }
        }

        next_frame().await;
    }
}
