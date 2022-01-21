# rui
Experimental Rust UI library for Audulus

Looks like this:

```Rust
    fn counter(start: usize) -> impl View {
        state(start, |count: State<usize>| {
            let count2 = count.clone();
            let value_string = format!("value: {:?}", *count.get());
            stack3(
                text(value_string.as_str()),
                button("increment", move || {
                    *count.get() += 1;
                }),
                button("decrement", move || {
                    *count2.get() -= 1;
                })
            )
        })
    }
````
