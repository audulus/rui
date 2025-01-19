use rui::*;

fn main() {
    state(
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
    )
    .run()
}
