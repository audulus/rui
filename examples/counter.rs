use rui::*;

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                text(&format!("{}", cx[count])).padding(Auto),
                button2(text("increment"), move |cx| {
                    *cx.get_mut(count) += 1
                })
                .padding(Auto),
            ))
        },
    ));
}
