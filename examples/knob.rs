use rui::*;

#[derive(Clone)]
struct MyState {
    value: f32,
}

fn main() {
    rui(state(MyState { value: 0.0 }, |state| {
        vstack((
            text(&format!("value: {:?}", state.get().value)).padding(Auto),
            knob(bind_no_copy!(state, MyState, value, f32)).padding(Auto),
        ))
    }));
}
