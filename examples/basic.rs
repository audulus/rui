use rui::*;

fn main() {
    rui(vstack((
        "This is a test.",
        rectangle().flex(),
        "This is another test.",
        rectangle().flex(),
    )));
}
