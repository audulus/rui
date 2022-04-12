use rui::*;

#[derive(Clone)]
struct MyState {
    value: f32,
}

fn main() {
    rui(state(
        || MyState { value: 0.0 },
        |state, cx| {
            let value = cx[state].value;
            vstack((
                text(&format!("value: {}", value)).padding(Auto),
                knob2(value, move |cx, v| cx[state].value = v).padding(Auto),
            ))
        },
    ));
}
