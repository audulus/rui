use rui::*;

fn main() {
    vstack((
        "This is a test.",
        rectangle().flex(),
        "This is another test.",
        rectangle().flex(),
    ))
    .run()
}
