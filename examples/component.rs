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

/// A component with private state of its own.
#[derive(Clone)]
struct ClickCounter;

impl Component for ClickCounter {
    fn body(&self) -> impl View {
        state(
            || 0,
            |clicks, cx| {
                button(format!("clicked {} times", cx[clicks]), move |cx| {
                    cx[clicks] += 1
                })
                .padding(Auto)
            },
        )
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
                ClickCounter,
                ClickCounter,
            ))
        },
    )
    .run()
}
