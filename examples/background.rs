use rui::*;

fn main() {
    rui(vstack! {
        text("this is a test")
            .padding(Auto)
            .background(
                rectangle(5.0)
                    .color(AZURE_HIGHLIGHT)
            )
    });
}
