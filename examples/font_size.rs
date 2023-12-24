use rui::*;

fn main() {
    state(
        || 0.0,
        |size, cx| {
            let s = (cx[size] * 100.0) as u32;
            vstack((
                "58".font_size(s),
                format!("font size: {}", s),
                hslider(size),
            ))
        },
    )
    .run()
}
