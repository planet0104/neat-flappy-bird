// #![windows_subsystem = "windows"]
mod game;
use game::Game;
use mengine::*;
use game::{GAME_WIDTH, GAME_HEIGHT};

impl State for game::Game {
    fn new(window: &mut Window) -> Self {
        let mut game = Game::new(window);
        game.start(window);
        game
    }

    fn update(&mut self, window: &mut Window){
        self.update(window);
    }

    fn event(&mut self, event: Event, window: &mut Window){
        // log(format!("{:?}", event));
        match event {
            Event::KeyUp(key) => {
                match key.to_lowercase().as_str(){
                    "f5" => self.reset(window),
                    "control" => self.key_ctrl_pressed = false,
                    _ => ()
                };
            }
            Event::KeyDown(key) => {
                match key.to_lowercase().as_str(){
                    "f5" => self.reset(window),
                    "control" => self.key_ctrl_pressed = true,
                    "1" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(60);
                        }
                    }
                    "2" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(120);
                        }
                    }
                    "3" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(180);
                        }
                    }
                    "4" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(240);
                        }
                    }
                    "5" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(300);
                        }
                    }
                    "m" => {
                        if self.key_ctrl_pressed {
                            window.set_update_rate(1000);
                        }
                    }
                    _ => ()
                };
            }
            _ => (),
        };
    }

    fn draw(&mut self, g: &mut Graphics) -> Result<(), String> {
        g.clear_rect(&[255, 255, 255, 255], 0.0, 0.0, GAME_WIDTH, GAME_HEIGHT);
        self.draw(g)?;
        Ok(())
    }
}

fn main() {
    run::<Game>(
        "NEAT Flappy Bird",
        GAME_WIDTH,
        GAME_HEIGHT,
        Settings {
            font_file: Some("wqy-micro-hei.ttf"),
            icon_path: Some("favicon.ico"),
            // auto_scale: true,
            // window_size: Some((100., 200.)),
            show_ups_fps: true,
            ..Default::default()
        },
    );
}
