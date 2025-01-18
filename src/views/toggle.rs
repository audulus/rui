use crate::*;

/// A toggle switch.
pub fn toggle(on: impl Binding<bool>) -> impl View {
    let width = 35.0f32;
    let edge = 1.0f32;
    let double_edge = edge * 2.0;
    let height = 20.0f32;

    state(
        || 0.0f32,
        move |animation: StateHandle<f32>, cx| {
            let b = *on.get(cx);
            zstack((
                rectangle()
                    .color(if b {
                        AZURE_HIGHLIGHT
                    } else {
                        CONTROL_BACKGROUND
                    })
                    .corner_radius(10.0)
                    .size([width, 20.0])
                    .tap(move |cx| on.with_mut(cx, |b| *b = !*b)),
                circle()
                    .color(MEDIUM_GRAY)
                    .size([height - double_edge, height - double_edge])
                    .offset([cx[animation] * (width - height) + edge, edge]),
            ))
            .anim(move |cx, dt| {
                let target = if b { 1.0 } else { 0.0 };
                let delta = target - cx[animation];
                cx[animation] += delta * 15.0 * dt;
                if delta.abs() < 0.01 {
                    cx[animation] = target;
                }
            })
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
        let s = StateHandle::<bool>::new(cx.view_id(&path));
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
