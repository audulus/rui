use rui::*;

fn main() {
    rui(zstack((
        text("This is a test."),
        circle().color(RED_HIGHLIGHT).padding(Auto),
    )));
}
