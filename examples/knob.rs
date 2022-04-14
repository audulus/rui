use rui::*;

#[derive(Clone, Debug, Default)]
struct MyState {
    x: f32,
}

fn main() {
    rui(state(MyState::default, |state, cx| {
        vstack((
            format!("value: {:?}", cx[state]).padding(Auto),
            knob_v(cx[state].x * 0.01, move |v, cx| cx[state].x = v * 100.0).padding(Auto),
        ))
    }));
}
