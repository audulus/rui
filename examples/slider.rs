use rui::*;

#[derive(Default)]
struct MyState {
    value: f32,
}

/// A slider with a value.
fn my_slider(s: State<f32>) -> impl View {
    get_cx(move |cx| {
        vstack((
            cx[s].font_size(10).padding(Auto),
            hslider(s).thumb_color(RED_HIGHLIGHT).padding(Auto),
        ))
    })
}

fn main() {
    rui(state(MyState::default, |state, cx| {
        map(
            cx[state].value,
            move |v, cx| cx[state].value = v,
            |s, _| my_slider(s),
        )
    }));
}
