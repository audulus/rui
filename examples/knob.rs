use rui::*;

#[derive(Clone, Debug, Default)]
struct MyState {
    x: f32,
}

make_lens!(MyLens, MyState, f32, x);

fn main() {
    rui(state(MyState::default, |state, cx| {
        vstack((
            text(&format!("value: {:?}", cx[state])).padding(Auto),
            knob(bind(state, MyLens {})).padding(Auto),
        ))
    }));
}
