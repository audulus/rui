use rui::*;

#[derive(Clone)]
struct MyState {
    value: f32,
}

fn main() {
    rui(state(
        || MyState { value: 0.0 },
        |state, cx| {
            let value = state.get().value;
            vstack((
                text(&format!("value: {}", value)).padding(Auto),
                knob2(value, move |v| state.with_mut(|x| x.value = v)).padding(Auto),
            ))
        },
    ));
}
