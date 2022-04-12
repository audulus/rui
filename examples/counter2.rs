use rui::*;

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                text(&format!("{}", count.get())).padding(Auto),
                button(text("increment"), move |_cx| {
                    count.with_mut(|x| *x += 1);
                })
                .padding(Auto),
                button(text("decrement"), move |_cx| {
                    count.with_mut(|x| *x -= 1);
                })
                .padding(Auto),
            ))
        },
    ));
}
