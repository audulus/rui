use rui::*;

fn main() {
    /*
    let s = state2(
        || 1,
        |count| {
            circle().tap2(|| *count += 1)
        },
    );*/

    let mut count = 0;
    let countref = &mut count;
    circle().tap2(|| *countref += 1);
}
