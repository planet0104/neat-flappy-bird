// #![windows_subsystem = "windows"]
#[macro_use]
extern crate serde_json;
mod ctx;
mod game;
mod neat;
mod timer;

use handlebars::Handlebars;
use serde_json::Value;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use timer::Timer;
use web_view::*;

fn main() -> WVResult {
    let reg = Handlebars::new();
    let html = String::from_utf8(include_bytes!("../index.html").to_vec()).unwrap();
    let context = json!({
        "game_width": game::GAME_WIDTH,
        "game_height": game::GAME_HEIGHT,
        "images": [
            {"id": game::IMG_BACKGROUND, "src": format!("data:image/png;base64,{}", base64::encode(include_bytes!("../images/background.png").as_ref()))},
            {"id": game::IMG_BIRD, "src": format!("data:image/png;base64,{}", base64::encode(include_bytes!("../images/bird.png").as_ref()))},
            {"id": game::IMG_PIPEBOTTOM, "src": format!("data:image/png;base64,{}", base64::encode(include_bytes!("../images/pipebottom.png").as_ref()))},
            {"id": game::IMG_PIPETOP, "src": format!("data:image/png;base64,{}", base64::encode(include_bytes!("../images/pipetop.png").as_ref()))}
        ],
    });
    let rendered = reg.render_template(&html, &context).unwrap();

    let game = Arc::new(Mutex::new(game::Game::default()));
    let (sender, receiver) = channel();
    let tgame = game.clone();
    thread::spawn(move || {
        let sleep_micros = Duration::from_micros(10);
        let mut timer = Timer::new(60);
        loop {
            if let Ok(fps) = receiver.try_recv() {
                timer = Timer::new(fps);
            }
            if timer.ready_for_next_frame() {
                tgame.lock().expect("线程中game.lock失败").update();
            }
            //一些延迟降低CPU使用率
            thread::sleep(sleep_micros);
        }
    });

    let webview = web_view::builder()
        .title("NEAT Flappy Bird")
        .content(Content::Html(rendered))
        .size(500, 560)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            // println!("invoke_handler:{}", arg);
            let mut game = game.lock().expect("invoke_handler lock失败");
            if !game.has_handle() {
                game.set_handle(webview.handle());
            }
            let json: Result<Value, serde_json::error::Error> = serde_json::from_str(arg);
            match json {
                Ok(v) => match v["cmd"].as_str() {
                    Some("log") => println!("webview:{:?}", v["info"].as_str()),
                    Some("animation-frame") => {
                        game.draw()?;
                    }
                    Some("fps") => {
                        sender.send(v["info"].as_u64().unwrap()).unwrap();
                    }
                    Some("reset") => {
                        game.reset();
                        sender.send(60).unwrap();
                    }
                    Some("onload") => {
                        game.start();
                    }
                    _ => println!("未定义命令:{}", arg),
                },
                Err(err) => {
                    println!("命令解析失败:{:?}", err);
                }
            };
            Ok(())
        })
        .build()?;
    webview.run()
}
