use rui::*;

fn main() {
    rui(state(
        || 1,
        |count| {
            vstack((
                text(&format!("{}", count.get())).padding(Auto),
                circle().tap2(move || count.with_mut(|x| *x += 1))
                .padding(Auto),
            ))
        },
    ));
}
