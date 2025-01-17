use rui::*;

struct MyState {
    view: AnyView,
}

impl Default for MyState {
    fn default() -> Self {
        MyState {
            view: any_view(text("Test")),
        }
    }
}

fn main() {
    state(MyState::default, move |s, cx| cx[s].view).run()
}
