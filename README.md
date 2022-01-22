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
```

- Encode UI in types to ensure stable identity.
- Use immediate mode initially, then optimize to reduce redraw later.
- Use a forthcoming rust port of [vger](https://github.com/audulus/vger) for rendering.

## References

[Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html)

[Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html)
