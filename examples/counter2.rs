use rui::*;

fn main() {
    rui(state2(
        || 1,
        |count| {
            vstack((
                text(&format!("{}", *count)).padding(Auto),
                circle().tap2(|| println!("tapped"))
                .padding(Auto),
            ))
        },
    ));
}
