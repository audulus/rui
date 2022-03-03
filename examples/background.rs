use rui::*;

fn main() {
    rui(text("this is a test")
            .padding(Auto)
            .background(
                rectangle()
                    .corner_radius(5.0)
                    .color(AZURE_HIGHLIGHT)
            )
    );
}
