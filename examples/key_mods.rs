use rui::*;

fn main() {
    rui(hstack((
        zstack((
            circle()
                .color(RED_HIGHLIGHT.alpha(0.8))
                .tap(|cx| println!("tapped circle, key modifiers state: {:?}", cx.key_mods))
                .padding(Auto),
            "Tap (inside circle)",
        )),
        zstack((
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT_BACKGROUND)
                .drag(|cx, delta, _state, _mouse_button| {
                    println!("dragging: {:?}, key modifiers state: {:?}", delta, cx.key_mods)
                })
                .padding(Auto),
            "Drag (inside rectangle)".padding(Auto),
        )),
        "Handle key pressed"
            .key(|cx, key| println!("key: {:?}, key modifiers state: {:?}", key, cx.key_mods))
            .padding(Auto),
    )));
}
