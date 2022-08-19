use rui::*;

fn main() {
    rui(hstack((
        text("This text is clipped.")
            // .offset([0.0, 0.0])
            .clip(),
        text("This text isn't clipped."),
    )))
}
