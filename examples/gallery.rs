use rui::*;

fn slider_example() -> impl View {
    hstack((
        text("slider").padding(Auto),
        state(0.5, |s| hslider(s))
    ))
}

fn knob_example() -> impl View {
    hstack((
        text("knob").padding(Auto),
        state(0.5, |s| knob(s).size([30.0, 30.0]).padding(Auto))
    ))
}

fn toggle_example() -> impl View {
    hstack((
        text("toggle").padding(Auto),
        state(false, |b| toggle(b).size([30.0, 30.0]).padding(Auto))
    ))
}

fn main() {

    rui(vstack((
        text("rui widget gallery"),
        slider_example(),
        knob_example(),
        toggle_example()
    )).padding(Auto))

}