use macroquad::prelude::*;
use macroquad_platformer::*;
use macroquad_tiled as tiled;
use std::collections::{BTreeSet, HashSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

struct Player {
    collider: Actor,
    vel: Vec2,
}

struct Enemy {
    collider: Actor,
    vel: Vec2,
    dir: f32, // -1.0 or +1.0
    alive: bool,
}

// constants 

// Map dimensions (should match Tiled JSON: width/height)
const MAP_W: usize = 30;
const MAP_H: usize = 20;

// Tile size in pixels (should match Tiled JSON: tilewidth/tileheight)
const TILE_W: f32 = 16.0;
const TILE_H: f32 = 16.0;

// Actor sizes in pixels
const PLAYER_W: f32 = 12.0;
const PLAYER_H: f32 = 14.0;
const ENEMY_W: f32 = 12.0;
const ENEMY_H: f32 = 14.0;

// Movement tuning
const GRAVITY: f32 = 1200.0;
const MOVE_SPEED: f32 = 220.0;
const JUMP_SPEED: f32 = 420.0;
const ENEMY_SPEED: f32 = 80.0;

// Layer name in Tiled
const TILE_LAYER: &str = "Tile Layer 1";

// ---------- small helpers ----------

fn map_px_w() -> f32 {
    MAP_W as f32 * TILE_W
}
fn map_px_h() -> f32 {
    MAP_H as f32 * TILE_H
}

/// Convert (x,y) tile coords into a flat index in row-major order.
///
/// Important: this assumes the collision grid is stored as [y * MAP_W + x].
fn idx(x: usize, y: usize) -> usize {
    y * MAP_W + x
}

/// Extract tile ids that are actually used in a given layer. Helpful for debugging
/// and for quickly deciding which tile ids you treat as solid.
fn collect_used_tile_ids(map: &tiled::Map) -> BTreeSet<u32> {
    let mut used = BTreeSet::new();
    for (_x, _y, tile) in map.tiles(TILE_LAYER, None) {
        if let Some(t) = tile {
            used.insert(t.id);
        }
    }
    used
}

/// Build a static collision grid (Tile::Solid / Tile::Empty) from the tiled layer.
///
/// We build a vec with exact size MAP_W * MAP_H up-front, then fill it by iterating
/// the map tiles. This avoids any mismatch / ordering assumptions.
fn build_static_colliders(map: &tiled::Map, solid_ids: &HashSet<u32>) -> Vec<Tile> {
    let mut colliders = vec![Tile::Empty; MAP_W * MAP_H];

    for (x, y, tile) in map.tiles(TILE_LAYER, None) {
        let x = x as usize;
        let y = y as usize;

        // Defensive guard (in case the tiles iterator yields something unexpected)
        if x >= MAP_W || y >= MAP_H {
            continue;
        }

        let solid = tile
            .as_ref()
            .map(|t| solid_ids.contains(&t.id))
            .unwrap_or(false);

        colliders[idx(x, y)] = if solid { Tile::Solid } else { Tile::Empty };
    }

    colliders
}

/// Find enemy spawn positions.
/// We spawn on tiles that are empty, with solid tile directly below.
///
/// IMPORTANT: `World::add_actor(pos, w, h)` expects `pos` to be TOP-LEFT in world pixels.
/// So we compute a top-left spawn position that stands on the tile below.
fn find_spawn_points(colliders: &[Tile]) -> Vec<Vec2> {
    let mut spawns = Vec::new();

    for y in 0..MAP_H.saturating_sub(1) {
        for x in 0..MAP_W {
            let here = colliders[idx(x, y)];
            let below = colliders[idx(x, y + 1)];

            if matches!(here, Tile::Empty) && matches!(below, Tile::Solid) {
                // Center horizontally in the tile, but keep top-left coordinate for the actor.
                let px = x as f32 * TILE_W + (TILE_W - ENEMY_W) * 0.5;
                // Place the actor so its bottom touches the bottom of tile (x, y).
                // (y+1)*TILE_H is the bottom edge of the empty tile at row y.
                let py = (y as f32 + 1.0) * TILE_H - ENEMY_H;

                spawns.push(vec2(px, py));
            }
        }
    }

    spawns
}

/// Deterministic enemy spawn: pick every Nth spawn point, up to some cap.
/// (Simple and reproducible; later you can move this into a Tiled object layer.)
fn spawn_enemies(world: &mut World, spawn_points: &[Vec2]) -> Vec<Enemy> {
    let mut enemies = Vec::new();

    let step = 20; // tune density
    let max_enemies = 6;

    for (k, &sp) in spawn_points
        .iter()
        .step_by(step)
        .take(max_enemies)
        .enumerate()
    {
        enemies.push(Enemy {
            collider: world.add_actor(sp, ENEMY_W as i32, ENEMY_H as i32),
            vel: vec2(0.0, 0.0),
            dir: if k % 2 == 0 { 1.0 } else { -1.0 },
            alive: true,
        });
    }

    enemies
}

/// Build a new platformer world + actors for a fresh run.
fn reset_run(colliders: &[Tile], spawn_points: &[Vec2]) -> (World, Player, Vec<Enemy>, u32) {
    let mut world = World::new();
    world.add_static_tiled_layer(colliders.to_vec(), TILE_W, TILE_H, MAP_W, 1);

    // Player spawn (top-left coords).
    let player = Player {
        collider: world.add_actor(vec2(32.0, 32.0), PLAYER_W as i32, PLAYER_H as i32),
        vel: vec2(0.0, 0.0),
    };

    let enemies = spawn_enemies(&mut world, spawn_points);

    let score = 0;
    (world, player, enemies, score)
}

// ---------- main ----------

#[macroquad::main("My game")]
async fn main() {
    // --- load assets ---
    let tileset_texture: Texture2D = load_texture("assets/maps/sheet.png").await.unwrap();
    tileset_texture.set_filter(FilterMode::Nearest);

    let map_json = load_string("assets/maps/map.json").await.unwrap();
    let sheet_tsj = load_string("assets/maps/sheet.tsj").await.unwrap();

    let textures = &[("sheet.png", tileset_texture)];
    let external_tilesets = &[("sheet.tsj", sheet_tsj.as_str())];

    let tiled_map = tiled::load_map(&map_json, textures, external_tilesets).unwrap();

    // --- debug: discover which tiles are used ---
    let used = collect_used_tile_ids(&tiled_map);
    println!("Used tile ids in {TILE_LAYER}: {:?}", used);

    // treat all placed tiles as solid for now.
    let solid_ids: HashSet<u32> = used.into_iter().collect();

    // --- build collision grid and spawn points ---
    let static_colliders = build_static_colliders(&tiled_map, &solid_ids);
    let spawn_points = find_spawn_points(&static_colliders);

    // --- camera in world-space ---
    let mut world_camera =
        Camera2D::from_display_rect(Rect::new(0.0, map_px_h(), map_px_w(), -map_px_h()));

    // --- game state + run state ---
    let mut game_state = GameState::MainMenu;

    // “run state” (world/actors) — initialize once, but reset cleanly on restart.
    let (mut world, mut player, mut enemies, mut score) =
        reset_run(&static_colliders, &spawn_points);

    loop {
        let dt = get_frame_time();
        clear_background(BLACK);

        match game_state {
            GameState::MainMenu => {
                // UI should be in screen-space
                set_default_camera();

                draw_text(
                    "Press ENTER to Start",
                    screen_width() / 2.0 - 130.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );

                if is_key_pressed(KeyCode::Enter) {
                    // Fresh run when starting (also guarantees enemies exist).
                    (world, player, enemies, score) = reset_run(&static_colliders, &spawn_points);
                    game_state = GameState::Playing;
                }
            }

            GameState::Playing => {
                // --- camera follow ---
                // actor_pos is top-left; target should be center.
                let p = world.actor_pos(player.collider);
                let player_center = p + vec2(PLAYER_W * 0.5, PLAYER_H * 0.5);

                // Clamp camera so you don’t scroll outside the map.
                world_camera.target = vec2(
                    clamp(
                        player_center.x,
                        map_px_w() / 4.0,
                        map_px_w() - map_px_w() / 4.0,
                    ),
                    clamp(
                        player_center.y,
                        map_px_h() / 4.0,
                        map_px_h() - map_px_h() / 4.0,
                    ),
                );

                set_camera(&world_camera);

                // --- draw map in world-space ---
                tiled_map.draw_tiles(
                    TILE_LAYER,
                    Rect::new(0.0, 0.0, map_px_w(), map_px_h()),
                    None,
                );

                // --- player physics + input ---
                let pos = world.actor_pos(player.collider);
                let on_ground = world.collide_check(player.collider, pos + vec2(0.0, 1.0));

                // Gravity only while airborne
                if !on_ground {
                    player.vel.y += GRAVITY * dt;
                } else if player.vel.y > 0.0 {
                    // If we hit the floor, kill downward velocity.
                    player.vel.y = 0.0;
                }

                // Horizontal input
                let mut dir = 0.0;
                if is_key_down(KeyCode::Right) {
                    dir += 1.0;
                }
                if is_key_down(KeyCode::Left) {
                    dir -= 1.0;
                }
                player.vel.x = dir * MOVE_SPEED;

                // Jump only when grounded
                if is_key_pressed(KeyCode::Space) && on_ground {
                    player.vel.y = -JUMP_SPEED;
                }

                world.move_h(player.collider, player.vel.x * dt);
                world.move_v(player.collider, player.vel.y * dt);

                // Debug draw player
                let p = world.actor_pos(player.collider);
                draw_rectangle(p.x, p.y, PLAYER_W, PLAYER_H, GREEN);

                // --- enemy movement / AI ---
                for e in &mut enemies {
                    if !e.alive {
                        continue;
                    }

                    let pos = world.actor_pos(e.collider);
                    let on_ground = world.collide_check(e.collider, pos + vec2(0.0, 1.0));

                    if !on_ground {
                        e.vel.y += GRAVITY * dt;
                    } else if e.vel.y > 0.0 {
                        e.vel.y = 0.0;
                    }

                    // Simple “turn around” behavior:
                    // - flip if wall immediately ahead
                    // - flip if no ground slightly ahead (ledge)
                    let ahead = vec2(e.dir * 6.0, 0.0);
                    let wall_ahead = world.collide_check(e.collider, pos + ahead);
                    let ground_ahead =
                        world.collide_check(e.collider, pos + ahead + vec2(0.0, 2.0));

                    if wall_ahead || !ground_ahead {
                        e.dir *= -1.0;
                    }

                    e.vel.x = e.dir * ENEMY_SPEED;

                    world.move_h(e.collider, e.vel.x * dt);
                    world.move_v(e.collider, e.vel.y * dt);
                }

                // --- stomp logic + scoring ---
                let player_pos = world.actor_pos(player.collider);
                let player_rect = Rect::new(player_pos.x, player_pos.y, PLAYER_W, PLAYER_H);

                for e in &mut enemies {
                    if !e.alive {
                        continue;
                    }

                    let ep = world.actor_pos(e.collider);
                    let enemy_rect = Rect::new(ep.x, ep.y, ENEMY_W, ENEMY_H);

                    if player_rect.overlaps(&enemy_rect) {
                        // Stomp test:
                        // player moving down AND player bottom is close to enemy top.
                        let player_bottom = player_pos.y + PLAYER_H;
                        let enemy_top = ep.y;

                        let stomping = player.vel.y > 0.0 && player_bottom <= enemy_top + 6.0;

                        if stomping {
                            e.alive = false;
                            score += 1;
                            // Bounce upward a bit
                            player.vel.y = -JUMP_SPEED * 0.7;
                        } else {
                            game_state = GameState::GameOver;
                        }
                    }
                }

                // Keep only living enemies
                enemies.retain(|e| e.alive);

                // Draw enemies
                for e in &enemies {
                    let ep = world.actor_pos(e.collider);
                    draw_rectangle(ep.x, ep.y, ENEMY_W, ENEMY_H, RED);
                }

                // --- UI overlay (screen-space) ---
                set_default_camera();
                draw_text("P: pause", 20.0, 30.0, 24.0, WHITE);
                draw_text(&format!("Score: {score}"), 20.0, 60.0, 24.0, WHITE);

                if is_key_pressed(KeyCode::P) {
                    game_state = GameState::Paused;
                }
            }

            GameState::Paused => {
                set_default_camera();
                draw_text("Paused (P to resume)", 20.0, 40.0, 30.0, WHITE);

                if is_key_pressed(KeyCode::P) {
                    game_state = GameState::Playing;
                }
            }

            GameState::GameOver => {
                set_default_camera();
                draw_text(
                    "Game Over! Press ENTER to Restart",
                    screen_width() / 2.0 - 200.0,
                    screen_height() / 2.0,
                    30.0,
                    WHITE,
                );

                if is_key_pressed(KeyCode::Enter) {
                    // Full reset: world + player + enemies + score
                    (world, player, enemies, score) = reset_run(&static_colliders, &spawn_points);
                    game_state = GameState::Playing;
                }
            }
        }

        next_frame().await;
    }
}
