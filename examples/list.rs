use rui::*;

struct MyData {
    id: i32,
    name: String,
}

fn main() {
    let data = vec![
        MyData {
            id: 0,
            name: "John".into(),
        },
        MyData {
            id: 1,
            name: "Paul".into(),
        },
        MyData {
            id: 2,
            name: "George".into(),
        },
        MyData {
            id: 3,
            name: "Ringo".into(),
        },
    ];

    rui(list(data.iter().map(|d| d.id).collect(), move |id| {
        // Linear search is not good. Use a better data structure for production code.
        let d = data.iter().find(|d| d.id == *id).unwrap();

        hstack((circle(), format!("{}", d.name)))
    }));
}
