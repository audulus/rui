use rui::*;

/// A component with plain props.
#[derive(Clone)]
struct Title {
    text: String,
}

impl Component for Title {
    fn body(&self) -> impl View {
        self.text.clone().font_size(24).padding(Auto)
    }
}

/// A component that edits a value owned by its parent, via a binding.
#[derive(Clone)]
struct Stepper<B> {
    label: &'static str,
    value: B,
}

impl<B: Binding<i32>> Component for Stepper<B> {
    fn body(&self) -> impl View {
        let value = self.value;
        hstack((
            self.label.padding(Auto),
            button("-", move |cx| value.with_mut(cx, |v| *v -= 1)),
            with_ref(value, |v| format!("{v}").padding(Auto)),
            button("+", move |cx| value.with_mut(cx, |v| *v += 1)),
        ))
    }
}

/// A component that declares state of its own.
#[derive(Clone)]
struct ClickCounter {
    label: &'static str,
}

impl StatefulComponent for ClickCounter {
    type State = u32;

    fn init(&self) -> u32 {
        0
    }

    fn body(self, clicks: u32, set: StateHandle<u32>) -> impl View {
        button(format!("{}: {clicks}", self.label), move |cx| cx[set] += 1).padding(Auto)
    }
}

/// State declared as fields with `#[component]`, like SwiftUI's `@State`.
/// The parent passes only the props (`title`, `detail`, `start_open`).
#[component]
#[derive(Clone)]
struct Disclosure {
    title: &'static str,
    detail: &'static str,
    start_open: bool,
    #[state = self.start_open]
    open: bool,
    #[state]
    times_toggled: u32,
}

impl Disclosure {
    fn body(self, state: DisclosureState, set: DisclosureSet) -> impl View {
        let arrow = if state.open { "v" } else { ">" };
        vstack((
            button(format!("{arrow} {}", self.title), move |cx| {
                cx[set.open] = !cx[set.open];
                cx[set.times_toggled] += 1;
            }),
            cond(
                state.open,
                format!("{} (toggled {} times)", self.detail, state.times_toggled).padding(Auto),
                EmptyView {},
            ),
        ))
    }
}

struct Model {
    a: i32,
    b: i32,
}

make_lens!(ModelA, Model, i32, a);
make_lens!(ModelB, Model, i32, b);

fn main() {
    state(
        || Model { a: 1, b: 10 },
        |s, cx| {
            let total = cx[s].a + cx[s].b;
            vstack((
                Title {
                    text: "Components".into(),
                },
                Stepper {
                    label: "a",
                    value: bind(s, ModelA {}),
                },
                Stepper {
                    label: "b",
                    value: bind(s, ModelB {}),
                },
                format!("a + b = {total}").padding(Auto),
                ClickCounter { label: "left" },
                ClickCounter { label: "right" },
                Disclosure {
                    title: "Details",
                    detail: "State declared on the component.",
                    start_open: true,
                },
            ))
        },
    )
    .run()
}
