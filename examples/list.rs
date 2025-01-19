use rui::*;

fn main() {
    let data = vec!["John", "Paul", "George", "Ringo"];

    let ids = (0usize..data.len()).collect();

    list(ids, move |id| hstack((circle(), data[*id].to_string()))).run()
}
