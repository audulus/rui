use rui::*;

fn main() {
    rui(state(|| false, |s, _| toggle(s)));
}
