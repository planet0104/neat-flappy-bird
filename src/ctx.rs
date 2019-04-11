use web_view::{WVResult, Handle};

pub fn clear_rect(handle:&Handle<()>, x:i32, y:i32, width:i32, height:i32) -> WVResult{
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.clearRect({},{},{},{});", x, y, width, height)) })
}

pub fn draw_image(handle:&Handle<()>, image_id:&str, x:i32, y:i32, size:Option<(i32, i32)>) -> WVResult{
    let image = format!("window.images.get('{}')", image_id);
    if let Some((width, height)) = size{
        handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.drawImage({},{},{},{},{});", image, x, y, width, height)) })
    }else{
        handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.drawImage({},{},{});", image, x, y)) })
    }
}

pub fn fill_style(handle:&Handle<()>, style:&str) -> WVResult{
    let style = style.to_string();
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.fillStyle='{}';", style)) })
}

pub fn stroke_style(handle:&Handle<()>, style:&str) -> WVResult{
    let style = style.to_string();
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.strokeStyle='{}';", style)) })
}

pub fn save(handle:&Handle<()>) -> WVResult{
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.save();")) })
}

pub fn translate(handle:&Handle<()>, x:i32, y:i32) -> WVResult{
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.translate({},{});", x, y)) })
}

pub fn rotate(handle:&Handle<()>, d:f64) -> WVResult{
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.rotate({});", d)) })
}

pub fn restore(handle:&Handle<()>) -> WVResult{
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.restore();")) })
}

pub fn font(handle:&Handle<()>, font:&str) -> WVResult{
    let font = font.to_string();
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.font='{}';", font)) })
}

pub fn fill_text(handle:&Handle<()>, text:&str, x:i32, y:i32) -> WVResult{
    let text = text.to_string();
    handle.dispatch(move |webview| { webview.eval(&format!("window.ctx.fillText('{}', {}, {});", text, x, y)) })
}


