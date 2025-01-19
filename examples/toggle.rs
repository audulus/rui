use rui::*;

fn main() {
    state(|| false, |s, _| toggle(s)).run()
}
