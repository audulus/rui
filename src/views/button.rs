use crate::*;
use accesskit::Role;

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

#[derive(Default)]
struct ButtonState {
    hovered: bool,
    down: bool,
}

/// Calls a function when the button is tapped.
pub fn button<A: 'static, F: Fn(&mut Context) -> A + 'static + Clone>(
    view: impl View + Clone,
    f: F,
) -> impl View {
    state(
        || ButtonState::default(),
        move |s, cx| {
            let f = f.clone();
            view.clone()
                .padding(Auto)
                .background(
                    rectangle()
                        .corner_radius(BUTTON_CORNER_RADIUS)
                        .color(if cx[s].down {
                            BUTTON_DOWN_COLOR
                        } else if cx[s].hovered {
                            BUTTON_HOVER_COLOR
                        } else {
                            BUTTON_BACKGROUND_COLOR
                        }),
                )
                .tap_with_info(move |cx, info| match info.state {
                    TouchState::Begin => {
                        cx[s].down = true;
                    }
                    TouchState::End => {
                        cx[s].down = false;
                        if cx[s].hovered {
                            f(cx);
                        }
                    }
                })
                .hover(move |cx, inside| {
                    cx[s].hovered = inside;
                })
                .role(Role::Button)
        },
    )
}

/// Version of button which emits an action directly instead of taking a callback.
pub fn button_a<A: Clone + 'static>(view: impl View + Clone, action: A) -> impl View {
    button(view, move |_| action.clone())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_button() {
        let mut cx = Context::new();

        let ui = state(
            || false,
            |pushed, _| {
                button("button", move |cx| {
                    *pushed.get_mut(cx) = true;
                })
            },
        );
        let sz = [100.0, 100.0].into();

        let mut path = vec![0];
        let button_sz = ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::new(LocalPoint::zero(), [90.0, 90.0].into()),
            },
        );
        assert!(path.len() == 1);

        assert_eq!(button_sz, sz);
        let s = StateHandle::<bool>::new(cx.view_id(&path), &mut cx);
        assert!(!*s.get(&cx));

        let events = [
            Event::TouchBegin {
                id: 0,
                position: [50.0, 50.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [50.0, 50.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut cx, &mut actions);
        }

        let vid = cx.view_id(&path);
        assert!(cx.state_map.contains_key(&vid));

        // State should have changed.
        assert!(*s.get(&cx));
    }
}
