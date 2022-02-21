use rui::*;

fn main() {
    rui(vstack! {
        circle(Paint::Color(RED_HIGHLIGHT));
        rectangle(5.0)
    });
}
