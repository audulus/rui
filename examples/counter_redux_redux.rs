use rui::*;

// This example shows how to nest a redux inside another redux.

#[derive(Clone)]
enum Action {
    Increment,
    None,
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
        Action::None => (),
    }
}

#[derive(Clone)]
enum LocalAction {
    Increment,
}

struct LocalState {
    count: i32,
}

fn reduce_local(state: &mut LocalState, action: &LocalAction) -> Action {
    match action {
        LocalAction::Increment => {
            state.count += 1;
            if state.count == 5 {
                state.count = 0;
                Action::Increment
            } else {
                Action::None
            }
        }
    }
}

fn main() {
    rui(redux(AppState::new, reduce, |app_state| {
        vstack((
            format!("{}", app_state.count).padding(Auto),
            redux(|| LocalState{ count: 0 }, reduce_local, |_| {
                button_a("increment every 5 clicks", LocalAction::Increment)
            }),
        ))
    }));
}