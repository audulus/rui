use rui::*;

#[derive(Clone, Default)]
struct MyState {
    value: [f32; 2],
}

fn main() {
    rui(state(MyState::default, |state, cx| {
        vstack((
            text(&format!("value: {:?}", cx[state].value))
                .font_size(10)
                .padding(Auto),
            hslider(cx[state].value[0], move |cx, v| cx[state].value[0] = v)
                .thumb_color(RED_HIGHLIGHT)
                .padding(Auto),
        ))
    }));
}
