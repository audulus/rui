use rui::*;

#[derive(Clone, Default)]
struct MyState {
    value: [f32; 2],
}

fn main() {
    rui(state(MyState::default, |state| {
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
