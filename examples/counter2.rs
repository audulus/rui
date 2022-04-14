use rui::*;

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                format!("{}", cx[count]).padding(Auto),
                button("increment", move |cx| {
                    cx[count] += 1;
                })
                .padding(Auto),
                button("decrement", move |cx| {
                    cx[count] -= 1;
                })
                .padding(Auto),
            ))
        },
    ));
}
