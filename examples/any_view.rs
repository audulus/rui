use rui::*;

fn main() {
    rui(list(vec![7, 42, 666], |i| {
        if *i == 7 {
            any_view(circle())
        } else {
            any_view(rectangle())
        }.padding(Auto)
    }));
}
