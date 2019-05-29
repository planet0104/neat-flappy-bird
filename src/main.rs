// #![windows_subsystem = "windows"]
mod game;
use game::Game;
use game::{GAME_HEIGHT, GAME_WIDTH};
use mengine::*;

fn main() {
    run::<Game>(
        "NEAT Flappy Bird",
        GAME_WIDTH,
        GAME_HEIGHT,
        Settings {
            icon_path: Some("favicon.ico"),
            auto_scale: true,
            window_size: Some((200., 200.)),
            show_ups_fps: true,
            ..Default::default()
        },
    );
}
