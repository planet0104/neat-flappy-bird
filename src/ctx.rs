use web_view::{Handle, WVResult};

pub fn clear_rect(handle: &Handle<()>, x: i32, y: i32, width: i32, height: i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!(
            "window.ctx.clearRect({},{},{},{});",
            x, y, width, height
        ))
    })
}

pub fn draw_image(
    handle: &Handle<()>,
    image_id: &str,
    x: i32,
    y: i32,
    size: Option<(i32, i32)>,
) -> WVResult {
    let image = format!("window.images.get('{}')", image_id);
    if let Some((width, height)) = size {
        handle.dispatch(move |webview| {
            webview.eval(&format!(
                "window.ctx.drawImage({},{},{},{},{});",
                image, x, y, width, height
            ))
        })
    } else {
        handle.dispatch(move |webview| {
            webview.eval(&format!("window.ctx.drawImage({},{},{});", image, x, y))
        })
    }
}

pub fn fill_style(handle: &Handle<()>, style: &str) -> WVResult {
    let style = style.to_string();
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.fillStyle='{}';", style)))
}

pub fn stroke_style(handle: &Handle<()>, style: &str) -> WVResult {
    let style = style.to_string();
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.strokeStyle='{}';", style)))
}

pub fn save(handle: &Handle<()>) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.save();")))
}

pub fn translate(handle: &Handle<()>, x: i32, y: i32) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.translate({},{});", x, y)))
}

pub fn rotate(handle: &Handle<()>, d: f64) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.rotate({});", d)))
}

pub fn restore(handle: &Handle<()>) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.restore();")))
}

pub fn begain_path(handle: &Handle<()>) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.beginPath();")))
}

pub fn stroke(handle: &Handle<()>) -> WVResult {
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.stroke();")))
}

pub fn font(handle: &Handle<()>, font: &str) -> WVResult {
    let font = font.to_string();
    handle.dispatch(move |webview| webview.eval(&format!("window.ctx.font='{}';", font)))
}

pub fn line_width(handle: &Handle<()>, width: i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.lineWidth={};", width))
    })
}

pub fn move_to(handle: &Handle<()>, x: i32, y:i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.moveTo({},{});", x, y))
    })
}

pub fn ellipse(handle: &Handle<()>, x: i32, y:i32, radius_x:i32, radius_y:i32, rotation:f64, start_angle:f64, end_angle: f64) -> WVResult {
    handle.dispatch(move |webview| {
        //void ctx.ellipse(x, y, radiusX, radiusY, rotation, startAngle, endAngle [, anticlockwise]);
        webview.eval(&format!("window.ctx.ellipse({},{},{},{},{},{},{});", x, y, radius_x, radius_y, rotation, start_angle, end_angle))
    })
}

pub fn fill_rect(handle: &Handle<()>, x: i32, y:i32, width:i32, height:i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.fillRect({},{},{},{});", x, y, width, height))
    })
}

pub fn stroke_rect(handle: &Handle<()>, x: i32, y:i32, width:i32, height:i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.strokeRect({},{},{},{});", x, y, width, height))
    })
}

pub fn line_to(handle: &Handle<()>, x: i32, y:i32) -> WVResult {
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.lineTo({},{});", x, y))
    })
}

pub fn fill_text(handle: &Handle<()>, text: &str, x: i32, y: i32) -> WVResult {
    let text = text.to_string();
    handle.dispatch(move |webview| {
        webview.eval(&format!("window.ctx.fillText('{}', {}, {});", text, x, y))
    })
}
