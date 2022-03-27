use rui::*;

fn main() {

    rui(vstack((
        text("rui widget gallery"),
        hstack((
            text("slider").padding(Auto),
            state(0.5, |s| hslider(s))
        )),
        hstack((
            text("knob").padding(Auto),
            state(0.5, |s| knob(s).size([30.0, 30.0]).padding(Auto))
        )),
        hstack((
            text("toggle").padding(Auto),
            state(false, |b| toggle(b).size([30.0, 30.0]).padding(Auto))
        )),
    )).padding(Auto))

}