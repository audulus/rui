use rui::*;

fn main() {
    rui(state(
        || 1,
        |count| {
            vstack((
                text(&format!("{}", count.get())).padding(Auto),
                button(text("increment"), move || {
                    count.with_mut(|x| *x += 1);
                })
                .padding(Auto),
                button(text("decrement"), move || {
                    count.with_mut(|x| *x -= 1);
                })
                .padding(Auto),
            ))
        },
    ));
}
