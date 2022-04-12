use rui::*;

#[derive(Clone, Debug)]
struct MyState {
    x: f32,
}

make_lens!(MyLens, MyState, f32, x);

fn main() {
    rui(state(
        || MyState{ x: 0.0 },
        |state, cx| {
            vstack((
                text(&format!("value: {:?}", cx[state])).padding(Auto),
                knob(bind(state, MyLens{})).padding(Auto),
            ))
        },
    ));
}
