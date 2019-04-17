// #![windows_subsystem = "windows"]
mod game;
mod neat;
use game::Game;

use quicksilver::{
    geom::Vector,
    graphics::Color,
    input::{ButtonState, Key},
    lifecycle::{run, Event, Settings, State, Window},
    Result,
};

impl State for game::Game {
    fn new() -> Result<Game> {
        let mut game = Game::default();
        game.start();
        Ok(game)
    }

    fn update(&mut self, window: &mut Window) -> Result<()> {
        self.update(window)
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        match event {
            &Event::Key(Key::F5, ButtonState::Released) => self.reset(),
            &Event::Key(Key::LControl, ButtonState::Released)
            | &Event::Key(Key::RControl, ButtonState::Released) => self.key_ctrl_pressed = false,
            &Event::Key(Key::RControl, ButtonState::Pressed)
            | &Event::Key(Key::LControl, ButtonState::Pressed) => self.key_ctrl_pressed = true,
            &Event::Key(Key::Key1, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1000. / 60.);
                }
            }
            &Event::Key(Key::Key2, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1000. / 60. / 2.);
                }
            }
            &Event::Key(Key::Key3, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1000. / 60. / 3.);
                }
            }
            &Event::Key(Key::Key4, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1000. / 60. / 4.);
                }
            }
            &Event::Key(Key::Key5, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1000. / 60. / 5.);
                }
            }
            &Event::Key(Key::M, ButtonState::Pressed) => {
                if self.key_ctrl_pressed {
                    window.set_update_rate(1.);
                }
            }
            _ => (),
        };
        Ok(())
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;
        self.draw(window)?;
        Ok(())
    }
}

fn main() {
    //update_rate: 1000. / 60.,
    run::<Game>(
        "NEAT Flappy Bird",
        Vector::new(game::GAME_WIDTH, game::GAME_HEIGHT),
        Settings {
            icon_path: Some("favicon.ico"),
            ..Default::default()
        },
    );
}
