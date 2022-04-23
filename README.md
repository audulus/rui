<p align="center">
<img src="rui.png" alt="logo" width="200"/>
</p>

# rui

![build status](https://github.com/audulus/rui/actions/workflows/rust.yml/badge.svg)

Experimental Rust UI library, inspired by SwiftUI. Early days, but some stuff already works.

rui is immediate mode (there is no retained tree of views), GPU rendered, updates reacitvely (when your state changes), and has richer layout options than other immediate mode UIs.

[discord server](https://discord.gg/JCVVBU3sCN)

- macOS ✅ 
- Windows ✅
- Linux ✅ 
- iOS (WIP: see https://github.com/audulus/rui-ios)

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
    rui(hstack((
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

fn main() {
    rui(state(MyState::default, |state, cx| {
        vstack((
            cx[state].value.font_size(10).padding(Auto),
            map(
                cx[state].value,
                move |v, cx| cx[state].value = v,
                |s, _| hslider(s).thumb_color(RED_HIGHLIGHT).padding(Auto),
            ),
        ))
    }));
}
```

<img src="screenshots/slider.png" alt="slider screenshot" style="width:50%;">

widget gallery (`cargo run --example gallery`):

<img src="screenshots/gallery.png" alt="widgets gallery screenshot" style="width:50%;">

## Goals

- Encode UI in types to ensure stable identity.
- Use immediate mode initially, then optimize to reduce redraw later.
- Use [vger-rs](https://github.com/audulus/vger-rs) for rendering.
- Minimal boilerplate.
- Good looking.
- No `unsafe`.
- Accessible.

## Optional Features

- `tao` - (*enabled by default*) use tao for windowing.
- `winit` - use winit for windowing.
- Use `default-features = false` if you are embedding rui.

## Why?

In the long term, I'd like to move Audulus over to Rust. After looking at other available UI options, it seemed best to implement something resembling the existing immediate mode UI system I already have working in Audulus, but better.

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
- ❌ layout feedback
- ✅ animation

## References

[Towards principled reactive UI](https://raphlinus.github.io/rust/druid/2020/09/25/principled-reactive-ui.html)

[Towards a unified theory of reactive UI](https://raphlinus.github.io/ui/druid/2019/11/22/reactive-ui.html)

[Flutter's Rendering Pipeline](https://www.youtube.com/watch?v=UUfXWzp0-DU)

[Static Types in SwiftUI](https://www.objc.io/blog/2019/11/05/static-types-in-swiftui/)

[How Layout Works in SwiftUI](https://www.hackingwithswift.com/books/ios-swiftui/how-layout-works-in-swiftui)
