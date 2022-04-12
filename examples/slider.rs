use rui::*;

#[derive(Clone, Default)]
struct MyState {
    value: f32,
}

fn main() {
    rui(state(MyState::default, |state, cx| {
        vstack((
            text(&format!("value: {:?}", cx[state].value))
                .font_size(10)
                .padding(Auto),
            hslider(cx[state].value, move |cx, v| cx[state].value = v)
                .thumb_color(RED_HIGHLIGHT)
                .padding(Auto),
        ))
    }));
}
