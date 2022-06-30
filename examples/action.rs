use rui::*;

struct MyAction {}

fn main() {
    rui(vstack((
        rectangle()
            .tap(|_| {
                println!("rect tapped");
                MyAction {}
            })
            .padding(Auto),
        text("tap the rectangle to send an action"),
    ))
    .handle(|_: &MyAction| println!("action received")));
}
