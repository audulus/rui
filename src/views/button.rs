use crate::*;
use accesskit::Role;

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

/// Calls a function when the button is tapped.
pub fn button<A: 'static, F: Fn(&mut Context) -> A + 'static + Clone>(
    view: impl View + Clone,
    f: F,
) -> impl View {
    state(
        || false,
        move |hovering, cx| {
            let f = f.clone();
            view.clone()
                .padding(Auto)
                .background(
                    rectangle()
                        .corner_radius(BUTTON_CORNER_RADIUS)
                        .color(BUTTON_BACKGROUND_COLOR),
                )
                .tap(move |cx| f(cx))
                .hover(|_, inside| {
                    println!("inside button: {}", inside);
                })
                .role(Role::Button)
        },
    )
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

        let button_sz = ui.layout(
            cx.root_id,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::new(LocalPoint::zero(), [90.0, 90.0].into()),
            },
        );

        assert_eq!(button_sz, sz);
        let s = StateHandle::<bool>::new(cx.root_id);
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
            ui.process(event, cx.root_id, &mut cx, &mut actions);
        }

        assert!(cx.state_map.contains_key(&cx.root_id));

        // State should have changed.
        assert!(*s.get(&cx));
    }
}
