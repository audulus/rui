use rui::*;

fn main() {
    rui(state(|| false, |s| toggle(s)));
}
