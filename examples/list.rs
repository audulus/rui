use rui::*;

fn main() {
    rui(list(vec![7, 42, 666], |i| {
        hstack((circle(), format!("{:?}", i)))
    }));
}
