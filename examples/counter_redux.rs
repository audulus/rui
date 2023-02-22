use rui::*;

#[derive(Clone)]
enum Action {
    Increment,
    Decrement,
}

struct AppState {
    count: i32,
}

impl AppState {
    fn new() -> Self {
        AppState { count: 1 }
    }
}

fn reduce(state: &mut AppState, action: &Action) {
    match action {
        Action::Increment => state.count += 1,
        Action::Decrement => state.count -= 1,
    }
}

fn main() {
    rui(redux(AppState::new, reduce, |state| {
        vstack((
            format!("{}", state.count).padding(Auto),
            button_a("increment", Action::Increment).padding(Auto),
            button_a("decrement", Action::Decrement).padding(Auto),
        ))
    }));
}
