use rui::*;

#[derive(Default)]
struct MyState {
    value: f32,
}

fn main() {
    rui(state(MyState::default, |state, cx|
        vstack((
            cx[state].value.font_size(10).padding(Auto),
            map(cx[state].value,
                move |v, cx| cx[state].value = v,
                |s, _| 
                    hslider(s)
                        .thumb_color(RED_HIGHLIGHT)
                        .padding(Auto))
            ),
        ))
    );
}
