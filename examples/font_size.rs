use rui::*;

fn main() {
    rui(state(
        || 0.0,
        |size, cx| {
            let s = (cx[size] * 100.0) as u32;
            vstack((
                text("58").font_size(s),
                text(&format!("font size: {}", s)),
                hslider(cx[size], move |cx, v| cx[size] = v),
            ))
        },
    ));
}
