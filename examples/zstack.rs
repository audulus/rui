use rui::*;

fn main() {
    rui(zstack((
        "This is a test.",
        circle().color(RED_HIGHLIGHT).padding(Auto),
    )));
}
