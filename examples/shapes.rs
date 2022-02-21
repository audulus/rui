use rui::*;

fn main() {
    rui(vstack! {
        circle()
            .color(RED_HIGHLIGHT)
            .padding();
        rectangle(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding()
    });
}
