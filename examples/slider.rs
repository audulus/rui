use rui::*;

#[derive(Clone)]
struct MyState {
    value: [f32; 2],
}

fn main() {
    rui(state(|| MyState { value: [0.0, 0.0] }, |state| {
        vstack((
            text(&format!("value: {:?}", state.get().value))
                .font_size(10)
                .padding(Auto),
            hslider(bind!(state, value[0]))
                .thumb_color(RED_HIGHLIGHT)
                .padding(Auto),
        ))
    }));
}
