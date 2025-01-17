use rui::*;

struct MyState {
    offset: f32,
}

impl Default for MyState {
    fn default() -> Self {
        MyState { offset: 100.0 }
    }
}
trait MyMod {
    fn my_animation(self) -> AnyView;
}

impl MyMod for AnyView {
    fn my_animation(self) -> AnyView {
        any_view(state(MyState::default, |s, cx| {
            self.offset(LocalOffset::new(0.0, cx[s].offset))
                .anim(move |cx, _| {
                    cx[s].offset *= 0.9;
                })
        }))
    }
}

fn my_text(name: &str) -> AnyView {
    let name = name.to_string();
    any_view(text(name.as_str()))
}

fn main() {
    hstack((my_text("without"), my_text("with").my_animation())).run()
}
