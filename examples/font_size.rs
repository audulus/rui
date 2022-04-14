use rui::*;

fn main() {
    rui(state(
        || 0.0,
        |size, cx| {
            let s = (cx[size] * 100.0) as u32;
            vstack((
                "58".font_size(s),
                format!("font size: {}", s),
                hslider(size),
            ))
        },
    ));
}
