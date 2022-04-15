use rui::*;

fn main() {
    rui(hstack((
        zstack((
            circle()
                .color(RED_HIGHLIGHT.alpha(0.8))
                .tap(|_, key_mods| println!("tapped circle, key modifiers state: {:?}", key_mods))
                .padding(Auto),
            "Tap (inside circle)",
        )),
        zstack((
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT_BACKGROUND)
                .drag(|_, delta, _state, key_mods| {
                    println!("dragging: {:?}, key modifiers state: {:?}", delta, key_mods)
                })
                .padding(Auto),
            "Drag (inside rectangle)".padding(Auto),
        )),
        "Handle key pressed"
            .key(|_, key, key_mods| println!("key: {:?}, key modifiers state: {:?}", key, key_mods))
            .padding(Auto),
    )));
}
