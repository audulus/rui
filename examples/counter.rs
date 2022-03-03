use rui::*;

fn main() {
    rui(state(1, |count| {
        vstack((
            text(&format!("{:?}", count.get())).padding(Auto),
            button("increment", move || {
                let value = count.get();
                count.set(value + 1);
            })
            .padding(Auto),
        ))
    }));
}
