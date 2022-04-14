use rui::*;

fn main() {
    rui("this is a test"
        .padding(Auto)
        .background(rectangle().corner_radius(5.0).color(AZURE_HIGHLIGHT)));
}
