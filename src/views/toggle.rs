use crate::*;

/// Toggle switch.
pub fn toggle(on: impl Binding<bool>) -> impl View {
    state(
        || (),
        move |_, cx| {
            let b = *on.get(cx);
            zstack((
                rectangle()
                    .color(if b {
                        AZURE_HIGHLIGHT_BACKGROUND
                    } else {
                        CONTROL_BACKGROUND
                    })
                    .corner_radius(10.0)
                    .size([40.0, 20.0])
                    .tap(move |cx| on.with_mut(cx, |b| *b = !*b)),
                circle()
                    .color(if b { AZURE_HIGHLIGHT } else { MEDIUM_GRAY })
                    .size([10.0, 10.0])
                    .offset([if b { 25.0 } else { 5.0 }, 5.0]),
            ))
        },
    )
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_toggle() {
        let mut cx = Context::new();

        let ui = state(|| false, |s, _| toggle(s));
        let sz = [40.0, 20.0].into();

        let mut path = vec![0];
        let knob_sz = ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );

        assert_eq!(knob_sz, sz);
        let s = StateHandle::<bool>::new(cx.view_id(&path), &mut cx);
        assert_eq!(*s.get(&cx), false);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: [10.0, 10.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [10.0, 10.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut cx, &mut actions);
        }

        // State should have changed.
        assert_eq!(*s.get(&cx), true);
    }
}
