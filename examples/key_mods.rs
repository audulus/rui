use rui::*;

fn main() {
    rui(circle()
        .color(RED_HIGHLIGHT.alpha(0.8))
        .key_mods(|_, mods| println!("key modifiers state: {:?}", mods))
        .padding(Auto));
}
