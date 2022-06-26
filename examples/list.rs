use rui::*;

fn main() {
    let data = vec!["John", "Paul", "George", "Ringo"];

    rui(list((0usize..data.len()).collect(), move |id| {
        hstack((circle(), format!("{}", &data[*id])))
    }));
}
