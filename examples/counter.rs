use rui::*;

fn main() {
    state(
        || 1,
        |count, _| {
            vstack((
                (*count).padding(Auto),
                button("increment", move |_| {
                    let mut c = count;
                    *c += 1;
                })
                .padding(Auto),
            ))
        },
    )
    .run()
}
