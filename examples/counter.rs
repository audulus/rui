use rui::*;

fn main() {
    rui(state(1, |count: State<usize>| {
        button(format!("{:?}", *count.get()).as_str(), move || {
            *count.get() += 1;
        })
    }));
}
