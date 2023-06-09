use rui::*;

// This example shows how to mix the redux style, with state local
// to a view.

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

fn main() {
    rui(redux(AppState::new, reduce, |app_state| {
        vstack((
            format!("{}", app_state.count).padding(Auto),
            state(|| 0, |handle, _| {
                button("increment every 5 clicks", move|cx| { 
                    cx[handle] += 1;
                    if cx[handle] == 5 {
                        cx[handle] = 0;
                        Action::Increment
                    } else {
                        Action::None
                    }
                })
            }),
        ))
    }));
}
