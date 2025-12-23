use macroquad::prelude::*;
use macroquad_tiled as tiled;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GameState {
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

#[macroquad::main("My game")]
async fn main() {
    // Any setup code goes here
    // Load the texture
    let mut game_state: GameState = GameState::MainMenu;
    let tileset_texture = match load_texture("assets/maps/sheet.png").await {
        Ok(tex) => tex,
        Err(err) => {
            eprintln!("Failed to load texture assets/maps/sheet.png: {err}");
            return; // clean exit from main()
        }
    };

    tileset_texture.set_filter(FilterMode::Nearest);

    // Load the map JSON
    let tiled_map_json = match load_string("assets/maps/map.json").await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to load tiled map assets/maps/map.json: {err}");
            return; // clean exit from main()
        }
    };

    let sheet_tsj = match load_string("assets/maps/sheet.tsj").await {
        Ok(s) => s,
        Err(err) => {
            eprintln!("Failed to load tileset assets/maps/sheet.tsj: {err}");
            return;
        }
    };

    let textures = &[("sheet.png", tileset_texture)];
    let external_tilesets = &[("sheet.tsj", sheet_tsj.as_str())];

    let tiled_map = match tiled::load_map(&tiled_map_json, textures, external_tilesets) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Failed to parse/load tiled map: {e:?}");
            return;
        }
    };

    loop {
        let _dt = get_frame_time();
        clear_background(BLACK);

        // Match game mode
        match game_state {
            GameState::MainMenu => {
                // Render main menu
                if is_key_pressed(KeyCode::Enter) {
                    game_state = GameState::Playing;
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
                // Lets draw the whole map full screen
                // Default macroquad camera is pixel perfect with (0, 0) in top left corner and (screen_width(), screen_height()) on bottom right
                let dest_rect = Rect::new(0., 0., screen_width(), screen_height());

                // And just draw our level!
                tiled_map.draw_tiles("Tile Layer 1", dest_rect, None);

                // Update and render game
            }
            GameState::Paused => {
                // Render paused screen
            }
            GameState::GameOver => {
                // Render game over screen
            }
        }

        next_frame().await
    }
}
