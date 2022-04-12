use rui::*;

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                text(&format!("{}", cx[count])).padding(Auto),
                button(text("increment"), move |cx| {
                    cx[count] += 1;
                })
                .padding(Auto),
                button(text("decrement"), move |cx| {
                    cx[count] -= 1;
                })
                .padding(Auto),
            ))
        },
    ));
}
