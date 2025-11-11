// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game;
mod solver;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![make_move, find_best_move])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
fn find_best_move(state: &str) -> (String, f64) {
    let state: u64 = state.parse().unwrap();
    let (best_move, _score) = solver::find_best_move(state);
    (best_move, _score)
}

#[tauri::command]
fn make_move(state: &str, direction: &str) -> (String, i64) {
    let state: u64 = state.parse().unwrap();

    let up = game::move_up(state);
    let down = game::move_down(state);
    let left = game::move_left(state);
    let right = game::move_right(state);

    if up == state && down == state && left == state && right == state {
        return (state.to_string(), -1);
    }

    let new_state = {
        game::spawn_new_random_tile(match direction {
            "up" => up,
            "down" => down,
            "left" => left,
            "right" => right,
            _ => state,
        })
    };

    let score = {
        match direction {
            "up" | "down" => game::game_score_vertical(state),
            "left" | "right" => game::game_score_horizontal(state),
            _ => 0,
        }
    };

    (new_state.to_string(), score as i64)
}
