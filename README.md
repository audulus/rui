# rui
Experimental Rust UI library for Audulus. "rui" is a temporary name. Early days, but some stuff already works.

obligatory Counter (`cargo run --example counter`):

```Rust
use rui::*;

fn main() {
    rui(state(1, |count: State<usize>| {
        vstack! {
            text(&format!("{:?}", *count.get()))
                .padding(Auto);
            button("increment", move || {
                *count.get() += 1;
            })
                .padding(Auto)
        }
    }));
}
```

![counter screenshot](screenshots/counter.png)

some shapes (`cargo run --example shapes`):

```rust
use rui::*;

fn main() {
    rui(vstack! {
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto);
        rectangle(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
    });
}
```

![shapes screenshot](screenshots/shapes.png)

- Encode UI in types to ensure stable identity.
- Use immediate mode initially, then optimize to reduce redraw later.
- Use a [vger-rs](https://github.com/audulus/vger-rs) for rendering.

## References

[Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html)

[Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html)

[Flutter's Rendering Pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU)

[Static Types in SwiftUI](https://www.objc.io/blog/2019/11/05/static-types-in-swiftui/)
