use rui::*;

fn main() {
    list(vec![7, 42], |i| {
        if *i == 7 {
            any_view(circle())
        } else {
            any_view(rectangle())
        }
        .padding(Auto)
    })
    .run()
}
