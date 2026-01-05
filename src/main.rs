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

/// Jump phase model (semantic, not time-based).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JumpPhase {
    Takeoff, // short, non-looping
    Air,     // rising/apex/falling decided from vy
    Landing, // short, non-looping
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimId {
    Idle,
    Walk,
    Run,
    Dash,
    JumpTakeoff,
    JumpRise,
    JumpApex,
    JumpFall,
    JumpLand,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AnimMode {
    Loop,
    Once,
}

/// Simple frame-based animation from a spritesheet.
struct SpriteAnim {
    texture: Texture2D,
    frames: Vec<Rect>,
    durations: Vec<f32>,
    t: f32,
    mode: AnimMode,
}

impl SpriteAnim {
    fn new(texture: Texture2D, frames: Vec<Rect>, durations: Vec<f32>, mode: AnimMode) -> Self {
        assert!(
            !frames.is_empty() && frames.len() == durations.len(),
            "frames and durations must be same non-zero length"
        );
        Self {
            texture,
            frames,
            durations,
            t: 0.0,
            mode,
        }
    }

    fn restart(&mut self) {
        self.t = 0.0;
    }

    fn update(&mut self, dt: f32) {
        self.t += dt;
    }

    fn total_duration(&self) -> f32 {
        self.durations.iter().sum::<f32>().max(0.0001)
    }

    fn is_finished(&self) -> bool {
        matches!(self.mode, AnimMode::Once) && self.t >= self.total_duration()
    }

    fn frame_index(&self) -> usize {
        let total = self.total_duration();

        let mut time = match self.mode {
            AnimMode::Loop => self.t % total,
            AnimMode::Once => self.t.min(total - 1e-6), // clamp inside range
        };

        for (i, d) in self.durations.iter().enumerate() {
            if time < *d {
                return i;
            }
            time -= *d;
        }
        self.frames.len() - 1
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

struct AnimSet {
    idle: SpriteAnim,
    walk: SpriteAnim,
    run: SpriteAnim,
    dash: SpriteAnim,
    jump_takeoff: SpriteAnim,
    jump_rise: SpriteAnim,
    jump_apex: SpriteAnim,
    jump_fall: SpriteAnim,
    jump_land: SpriteAnim,
}

struct Player {
    pos: Vec2,
    vel: Vec2,
    facing: f32, // 1.0 right, -1.0 left

    on_ground: bool,

    // User-selected ground mode (I/W/R)
    ground_mode: MoveState,

    // Current movement state (Dash/Jump overrides ground_mode)
    state: MoveState,

    // Jump phase (only meaningful while state == Jump)
    jump_phase: JumpPhase,
}

fn sheet_frames(frame_count: usize, frame_w: f32, frame_h: f32) -> Vec<Rect> {
    (0..frame_count)
        .map(|i| Rect::new(i as f32 * frame_w, 0.0, frame_w, frame_h))
        .collect()
}

fn restart_if_changed(anims: &mut AnimSet, prev: &mut AnimId, next: AnimId) {
    if *prev == next {
        return;
    }
    *prev = next;
    match next {
        AnimId::Idle => anims.idle.restart(),
        AnimId::Walk => anims.walk.restart(),
        AnimId::Run => anims.run.restart(),
        AnimId::Dash => anims.dash.restart(),
        AnimId::JumpTakeoff => anims.jump_takeoff.restart(),
        AnimId::JumpRise => anims.jump_rise.restart(),
        AnimId::JumpApex => anims.jump_apex.restart(),
        AnimId::JumpFall => anims.jump_fall.restart(),
        AnimId::JumpLand => anims.jump_land.restart(),
    }
}

fn anim_for(player: &Player) -> AnimId {
    match player.state {
        MoveState::Dash => AnimId::Dash,

        MoveState::Jump => match player.jump_phase {
            JumpPhase::Takeoff => AnimId::JumpTakeoff,
            JumpPhase::Landing => AnimId::JumpLand,
            JumpPhase::Air => {
                // Velocity-driven pose selection (variable airtime safe)
                let vy = player.vel.y;
                if vy < -50.0 {
                    AnimId::JumpRise
                } else if vy > 50.0 {
                    AnimId::JumpFall
                } else {
                    AnimId::JumpApex
                }
            }
        },

        _ => match player.ground_mode {
            MoveState::Idle => AnimId::Idle,
            MoveState::Walk => AnimId::Walk,
            MoveState::Run => AnimId::Run,
            _ => AnimId::Idle,
        },
    }
}

fn anim_mut<'a>(anims: &'a mut AnimSet, id: AnimId) -> &'a mut SpriteAnim {
    match id {
        AnimId::Idle => &mut anims.idle,
        AnimId::Walk => &mut anims.walk,
        AnimId::Run => &mut anims.run,
        AnimId::Dash => &mut anims.dash,
        AnimId::JumpTakeoff => &mut anims.jump_takeoff,
        AnimId::JumpRise => &mut anims.jump_rise,
        AnimId::JumpApex => &mut anims.jump_apex,
        AnimId::JumpFall => &mut anims.jump_fall,
        AnimId::JumpLand => &mut anims.jump_land,
    }
}

#[macroquad::main("Sprite Demo")]
async fn main() {
    let mut game_state = GameState::MainMenu;

    // === Assets (each state uses a different PNG) ==========================
    // Put these in /assets and ensure they get deployed with gh-pages.
    let idle_tex = load_texture("assets/idle.png").await.unwrap();
    let walk_tex = load_texture("assets/idle.png").await.unwrap();
    let run_tex = load_texture("assets/idle.png").await.unwrap();
    let dash_tex = load_texture("assets/idle.png").await.unwrap();

    let jt_tex = load_texture("assets/idle.png").await.unwrap();
    let jr_tex = load_texture("assets/idle.png").await.unwrap();
    let ja_tex = load_texture("assets/idle.png").await.unwrap();
    let jf_tex = load_texture("assets/idle.png").await.unwrap();
    let jl_tex = load_texture("assets/idle.png").await.unwrap();

    for t in [
        &idle_tex, &walk_tex, &run_tex, &dash_tex, &jt_tex, &jr_tex, &ja_tex, &jf_tex, &jl_tex,
    ] {
        t.set_filter(FilterMode::Nearest);
    }

    // each sheet is 1 row, frames are 32x32.
    let mut anims = AnimSet {
        idle: SpriteAnim::new(
            idle_tex,
            sheet_frames(5, 32.0, 32.0),
            vec![0.20; 5],
            AnimMode::Loop,
        ),
        walk: SpriteAnim::new(
            walk_tex,
            sheet_frames(4, 32.0, 32.0),
            vec![0.12; 4],
            AnimMode::Loop,
        ),
        run: SpriteAnim::new(
            run_tex,
            sheet_frames(4, 32.0, 32.0),
            vec![0.08; 4],
            AnimMode::Loop,
        ),
        dash: SpriteAnim::new(
            dash_tex,
            sheet_frames(3, 32.0, 32.0),
            vec![0.06; 3],
            AnimMode::Loop,
        ),

        // Jump phases: takeoff + landing are "Once". Air poses can be 1-frame "Loop" (held).
        jump_takeoff: SpriteAnim::new(
            jt_tex,
            sheet_frames(3, 32.0, 32.0),
            vec![0.06; 3],
            AnimMode::Once,
        ),
        jump_rise: SpriteAnim::new(
            jr_tex,
            sheet_frames(1, 32.0, 32.0),
            vec![1.0; 1],
            AnimMode::Loop,
        ),
        jump_apex: SpriteAnim::new(
            ja_tex,
            sheet_frames(1, 32.0, 32.0),
            vec![1.0; 1],
            AnimMode::Loop,
        ),
        jump_fall: SpriteAnim::new(
            jf_tex,
            sheet_frames(1, 32.0, 32.0),
            vec![1.0; 1],
            AnimMode::Loop,
        ),
        jump_land: SpriteAnim::new(
            jl_tex,
            sheet_frames(3, 32.0, 32.0),
            vec![0.06; 3],
            AnimMode::Once,
        ),
    };

    let scale = 6.0;
    let sprite_w = 32.0 * scale;
    let sprite_h = 32.0 * scale;

    let ground_y = screen_height() * 0.75 - sprite_h;
    let start_x = screen_width() * 0.5 - sprite_w * 0.5;

    let mut player = Player {
        pos: vec2(start_x, ground_y),
        vel: vec2(0.0, 0.0),
        facing: 1.0,
        on_ground: true,
        ground_mode: MoveState::Idle,
        state: MoveState::Idle,
        jump_phase: JumpPhase::Air, // irrelevant until Jump
    };

    let mut prev_anim = AnimId::Idle;

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                draw_text(
                    "Sprite Demo",
                    screen_width() * 0.5 - 100.0,
                    screen_height() * 0.4,
                    48.0,
                    WHITE,
                );
                draw_text(
                    "ENTER: start  |  ESC: quit",
                    screen_width() * 0.5 - 180.0,
                    screen_height() * 0.55,
                    28.0,
                    WHITE,
                );
                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::Demo;

                    player.pos = vec2(start_x, ground_y);
                    player.vel = vec2(0.0, 0.0);
                    player.facing = 1.0;
                    player.on_ground = true;
                    player.ground_mode = MoveState::Idle;
                    player.state = MoveState::Idle;
                    prev_anim = AnimId::Idle;
                    // restart all for consistency
                    anims.idle.restart();
                    anims.walk.restart();
                    anims.run.restart();
                    anims.dash.restart();
                    anims.jump_takeoff.restart();
                    anims.jump_rise.restart();
                    anims.jump_apex.restart();
                    anims.jump_fall.restart();
                    anims.jump_land.restart();
                }
                if is_key_pressed(KeyCode::Escape) {
                    std::process::exit(0);
                }
            }

            GameState::Demo => {
                // ---- UI + Controls -------------------------------------------------
                draw_text(
                    "I:Idle  W:Walk  R:Run  J:Jump  D:Dash    <-/->:face    ESC: menu",
                    20.0,
                    30.0,
                    22.0,
                    WHITE,
                );
                draw_text(
                    &format!(
                        "State: {:?}   GroundMode: {:?}   JumpPhase: {:?}",
                        player.state, player.ground_mode, player.jump_phase
                    ),
                    20.0,
                    60.0,
                    22.0,
                    YELLOW,
                );
                draw_text(&format!("vy: {:.1}", player.vel.y), 20.0, 90.0, 22.0, GREEN);

                if is_key_pressed(KeyCode::Escape) {
                    game_state = GameState::MainMenu;
                    continue;
                }

                // Facing direction (independent of movement mode)
                if is_key_down(KeyCode::Left) {
                    player.facing = -1.0;
                } else if is_key_down(KeyCode::Right) {
                    player.facing = 1.0;
                }

                // Ground-mode selection keys (only really used when not in Jump/Dash)
                if is_key_pressed(KeyCode::I) {
                    player.ground_mode = MoveState::Idle;
                    if player.on_ground
                        && player.state != MoveState::Dash
                        && player.state != MoveState::Jump
                    {
                        player.state = MoveState::Idle;
                    }
                }
                if is_key_pressed(KeyCode::W) {
                    player.ground_mode = MoveState::Walk;
                    if player.on_ground
                        && player.state != MoveState::Dash
                        && player.state != MoveState::Jump
                    {
                        player.state = MoveState::Walk;
                    }
                }
                if is_key_pressed(KeyCode::R) {
                    player.ground_mode = MoveState::Run;
                    if player.on_ground
                        && player.state != MoveState::Dash
                        && player.state != MoveState::Jump
                    {
                        player.state = MoveState::Run;
                    }
                }

                // Jump start (phase = Takeoff)
                if is_key_pressed(KeyCode::J) && player.on_ground {
                    player.state = MoveState::Jump;
                    player.jump_phase = JumpPhase::Takeoff;
                    player.on_ground = false;

                    // TODO: Variable jump height can be added by cutting vy on key release.
                    player.vel.y = -900.0;

                    anims.jump_takeoff.restart();
                }

                // Dash start (transient)
                if is_key_pressed(KeyCode::D) {
                    player.state = MoveState::Dash;
                    player.vel.x = player.facing * 1200.0;
                    anims.dash.restart();
                }

                // Optional: variable jump height (short hop vs full jump)
                // for shorter jump release J early!
                if is_key_released(KeyCode::J)
                    && player.state == MoveState::Jump
                    && player.vel.y < 0.0
                {
                    player.vel.y *= 0.45;
                }

                // ---- Physics -------------------------------------------------------
                let gravity = 2200.0;
                let (walk_speed, run_speed) = (250.0, 450.0);

                // Horizontal velocity logic depends on state
                match player.state {
                    MoveState::Dash => {
                        // decay dash; once it slows down, return to ground mode
                        player.vel.x *= 0.88;
                        if player.vel.x.abs() < 80.0 {
                            player.state = player.ground_mode;
                            if player.on_ground {
                                player.vel.x = 0.0;
                            }
                        }
                    }

                    MoveState::Jump => {
                        // Air control: follow ground_mode (walk/run) while airborne
                        let air_speed = match player.ground_mode {
                            MoveState::Run => run_speed * 0.6,
                            MoveState::Walk => walk_speed * 0.7,
                            _ => walk_speed * 0.5,
                        };
                        player.vel.x = player.facing * air_speed;
                    }

                    _ => {
                        // On ground: follow selected ground_mode
                        player.state = player.ground_mode;
                        player.vel.x = match player.ground_mode {
                            MoveState::Idle => 0.0,
                            MoveState::Walk => player.facing * walk_speed,
                            MoveState::Run => player.facing * run_speed,
                            _ => 0.0,
                        };
                    }
                }

                // Apply gravity if airborne
                if !player.on_ground {
                    player.vel.y += gravity * dt;
                }

                // Integrate
                player.pos += player.vel * dt;

                // --- Screen wrap (horizontal) -----------------------------------
                let screen_w = screen_width();

                // If sprite fully exits right -> appear on left
                if player.pos.x > screen_w {
                    player.pos.x = -sprite_w;
                }

                // If sprite fully exits left -> appear on right
                if player.pos.x + sprite_w < 0.0 {
                    player.pos.x = screen_w;
                }

                // Ground collision detection
                let mut just_landed = false;
                if player.pos.y >= ground_y {
                    if !player.on_ground {
                        just_landed = true;
                    }
                    player.pos.y = ground_y;
                    player.vel.y = 0.0;
                    player.on_ground = true;
                } else {
                    player.on_ground = false;
                }

                // ---- Jump phase transitions----------------------------
                if player.state == MoveState::Jump {
                    // If we just landed: play Landing once
                    if just_landed {
                        player.jump_phase = JumpPhase::Landing;
                        anims.jump_land.restart();
                    }

                    // While airborne (and not in takeoff/landing): phase = Air
                    if !player.on_ground && player.jump_phase != JumpPhase::Takeoff {
                        player.jump_phase = JumpPhase::Air;
                    }

                    // Takeoff ends when its non-looping anim ends -> switch to Air
                    if player.jump_phase == JumpPhase::Takeoff && anims.jump_takeoff.is_finished() {
                        player.jump_phase = JumpPhase::Air;
                    }

                    // Landing ends when its non-looping anim ends -> return to ground_mode
                    if player.jump_phase == JumpPhase::Landing && anims.jump_land.is_finished() {
                        player.state = player.ground_mode;
                        // jump_phase not used outside Jump, but keep it sane
                        player.jump_phase = JumpPhase::Air;
                    }
                }

                // ---- Draw ----------------------------------------------------------
                // Ground line
                draw_line(
                    0.0,
                    ground_y + sprite_h,
                    screen_width(),
                    ground_y + sprite_h,
                    2.0,
                    DARKGRAY,
                );

                let wanted = anim_for(&player);
                restart_if_changed(&mut anims, &mut prev_anim, wanted);

                // Update + draw only the current anim
                let flip_x = player.facing < 0.0;
                let a = anim_mut(&mut anims, wanted);
                a.update(dt);
                a.draw(player.pos, scale, flip_x);
            }
        }

        next_frame().await;
    }
}
