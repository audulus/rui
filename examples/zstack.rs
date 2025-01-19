use rui::*;

fn main() {
    zstack((
        "This is a test.",
        circle().color(RED_HIGHLIGHT).padding(Auto),
    ))
    .run()
}
