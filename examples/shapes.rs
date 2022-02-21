use rui::*;

fn main() {
    rui(vstack! {
        circle();
        rectangle(5.0)
    });
}
