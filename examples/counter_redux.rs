use rui::*;

#[derive(Clone)]
enum Action {
    Increment,
    Decrement
}

fn reduce(state: &mut i32, action: &Action) {
    match action {
        Action::Increment => *state += 1,
        Action::Decrement => *state -= 1,
    }
}

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                format!("{}", cx[count]).padding(Auto),
                button_a("increment", Action::Increment)
                .padding(Auto),
                button_a("decrement", Action::Decrement)
                .padding(Auto),
            )).handle(move |cx, action: &Action| {
                reduce(&mut cx[count], action)
            })
        },
    ));
}
