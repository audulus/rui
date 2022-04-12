use rui::*;

fn main() {
    rui(state(|| false, |s, cx| toggle(cx[s], move |cx, b| cx[s] = b)));
}
