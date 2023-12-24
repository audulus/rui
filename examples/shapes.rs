use rui::*;

fn main() {
    hstack((
        circle().color(RED_HIGHLIGHT).padding(Auto),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto),
    ))
    .run()
}
