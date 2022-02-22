use rui::*;

fn main() {
    rui(hstack! {
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto);
        rectangle(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
    });
}
