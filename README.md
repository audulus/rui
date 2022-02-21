# rui
Experimental Rust UI library for Audulus. "rui" is a temporary name. Early days, but some stuff already works.

obligatory Counter:

```Rust
use rui::*;

fn main() {
    rui(state(1, |count: State<usize>| {
        vstack! {
            text(format!("{:?}", *count.get()).as_str()).padding();
            button("increment", move || {
                *count.get() += 1;
            }).padding()
        }
    }));
}
```

![counter screenshot](screenshots/counter.png)

- Encode UI in types to ensure stable identity.
- Use immediate mode initially, then optimize to reduce redraw later.
- Use a [vger-rs](https://github.com/audulus/vger-rs) for rendering.

## References

[Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html)

[Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html)

[Flutter's Rendering Pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU)

[Static Types in SwiftUI](https://www.objc.io/blog/2019/11/05/static-types-in-swiftui/)
