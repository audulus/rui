<p align="center">
<img src="rui.png" alt="logo" width="200"/>
</p>

# rui

![build status](https://github.com/audulus/rui/actions/workflows/rust.yml/badge.svg)
[![dependency status](https://deps.rs/repo/github/audulus/rui/status.svg)](https://deps.rs/repo/github/audulus/rui)

Experimental Rust UI library, inspired by SwiftUI. Early days, but some stuff already works. rui will be used for a future version of [Audulus](http://audulus.com/)

rui is immediate mode (there is no retained tree of views), GPU rendered, updates reactively (when your state changes), and has richer layout options than other immediate mode UIs.

[discord server](https://discord.gg/JCVVBU3sCN)

- macOS ✅ 
- Windows ✅
- Linux ✅ 
- iOS ✅ (see https://github.com/audulus/rui-ios)
- wasm (WIP)

## Examples

obligatory Counter (`cargo run --example counter`):

```rust
use rui::*;

fn main() {
    rui(state(
        || 1,
        |count, cx| {
            vstack((
                cx[count].padding(Auto),
                button("increment", move |cx| {
                    cx[count] += 1;
                })
                .padding(Auto),
            ))
        },
    ));
}
```

<img src="screenshots/counter.png" alt="counter screenshot" style="width:50%;">

some shapes (`cargo run --example shapes`):

```rust
use rui::*;

fn main() {
    rui(vstack((
        circle()
            .color(RED_HIGHLIGHT)
            .padding(Auto),
        rectangle()
            .corner_radius(5.0)
            .color(AZURE_HIGHLIGHT)
            .padding(Auto)
    )));
}
```

<img src="screenshots/shapes.png" alt="shapes screenshot" style="width:50%;">

canvas for gpu drawing (`cargo run --example canvas`):

```rust
use rui::*;

fn main() {
    rui(canvas(|_, rect, vger| {
        vger.translate(rect.center() - LocalPoint::zero());

        let paint = vger.linear_gradient(
            [-100.0, -100.0],
            [100.0, 100.0],
            AZURE_HIGHLIGHT,
            RED_HIGHLIGHT,
            0.0,
        );

        let radius = 100.0;
        vger.fill_circle(LocalPoint::zero(), radius, paint);
    }));
}
```

<img src="screenshots/canvas.png" alt="canvas screenshot" style="width:50%;">

`slider` with `map` (`cargo run --example slider`):

```rust
use rui::*;

#[derive(Default)]
struct MyState {
    value: f32,
}

/// A slider with a value.
fn my_slider(s: impl Binding<f32>) -> impl View {
    with_ref(s, move |v| {
        vstack((
            v.to_string().font_size(10).padding(Auto),
            hslider(s).thumb_color(RED_HIGHLIGHT).padding(Auto),
        ))
    })
}

fn main() {
    rui(state(MyState::default, |state, cx| 
        map(
            cx[state].value,
            move |v, cx| cx[state].value = v,
            |s, _| my_slider(s),
        ),
    ));
}
```

<img src="screenshots/slider.png" alt="slider screenshot" style="width:50%;">

widget gallery (`cargo run --example gallery`):

<img src="screenshots/gallery.png" alt="widgets gallery screenshot" style="width:50%;">

## Goals

- Encode UI in types to ensure stable identity.
- Optimize to reduce redraw.
- Use [vger-rs](https://github.com/audulus/vger-rs) for rendering.
- Minimal boilerplate.
- Good looking.
- No `unsafe`.
- Accessibility for assistive technologies.

## Optional Features

- `winit` - (*enabled by default*) use winit for windowing.
- Use `default-features = false` if you are embedding rui (see https://github.com/audulus/rui-ios).

## Why?

In the long term, I'd like to move [Audulus](http://audulus.com/) over to Rust. After looking at other available UI options, it seemed best to implement something resembling the existing immediate mode UI system I already have working in Audulus, but better.

## Status

- ✅ basic shapes: circle, rounded rectangle
- ✅ basic gestures: tap, drag
- ✅ hstack/vstack
- ✅ text
- ✅ padding
- ✅ offsets
- ✅ state
- ✅ zstack
- ✅ canvas (GPU vector graphics with vger)
- ✅ bindings
- ✅ list
- ✅ sliders
- ✅ knobs
- ✅ editable text (still a bit rough)
- ✅ any_view (view type erasure)
- ✅ layout feedback
- ✅ animation
- ✅ UI unit testing

## References

[Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html)

[Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html)

[Flutter's Rendering Pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU)

[Static Types in SwiftUI](https://www.objc.io/blog/2019/11/05/static-types-in-swiftui/)

[How Layout Works in SwiftUI](https://www.hackingwithswift.com/books/ios-swiftui/how-layout-works-in-swiftui)

[Xilem: an architecture for UI in Rust](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html)
