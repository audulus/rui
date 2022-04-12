use rui::*;

fn main() {
    rui(state(|| false, |s, cx| toggle(s)));
}
