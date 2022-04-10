use rui::*;

fn main() {
    rui(state(
        || 0.0,
        |size| {
            let s = (size.get() * 100.0) as u32;
            vstack((
                text("58").font_size(s),
                text(&format!("font size: {}", s)),
                hslider(size),
            ))
        },
    ));
}
