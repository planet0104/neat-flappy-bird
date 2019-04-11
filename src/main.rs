#[macro_use]
extern crate serde_json;
use serde_json::Value;
use web_view::*;
use handlebars::Handlebars;

mod neat;
mod ctx;
mod game;

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

    let mut game = game::Game::default();

    let webview = web_view::builder()
        .title("NEAT Flappy Bird")
        .content(Content::Html(rendered))
        .size(500, 560)
        .resizable(false)
        .debug(true)
        .user_data(())
        .invoke_handler(|webview, arg| {
            // println!("invoke_handler:{}", arg);
            if !game.has_handle(){
                game.set_handle(webview.handle());
            }
            let json: Result<Value, serde_json::error::Error> = serde_json::from_str(arg);
            match json {
                Ok(v) => match v["cmd"].as_str() {
                    Some("log") => println!("webview:{:?}", v["info"].as_str()),
                    Some("animation-frame") => {
                        // println!("animation-frame");
                        game.update();
                        game.draw()?;
                    },
                    Some("onload") => {
                        println!("加载完成");
                        game.start();
                    },
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
