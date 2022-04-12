use rui::*;

fn main() {
    rui(state(
        || 0.0,
        |state, cx| {
            let value = cx[state];
            vstack((
                text(&format!("value: {}", value)).padding(Auto),
                knob(state).padding(Auto),
            ))
        },
    ));
}
