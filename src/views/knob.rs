use crate::*;

const THETA_MIN: f32 = 3.0 / 2.0 * std::f32::consts::PI;
const THETA_MAX: f32 = 7.0 / 2.0 * std::f32::consts::PI;

fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (1.0 - x) * a + x * b
}

/// Knob for controlling a 0 to 1 floating point parameter.
pub fn knob(value: impl Binding<f32>) -> impl View {
    zstack((
        circle()
            .color(CLEAR_COLOR)
            .drag_s(value, move |v, delta, _, _| {
                *v = (*v + (delta.x + delta.y) / 400.0).clamp(0.0, 1.0)
            })
            .grab_cursor(),
        canvas(move |cx, sz, vger| {
            let c = sz.center();
            let r = sz.width().min(sz.height()) / 2.0;

            let paint = vger.color_paint(CONTROL_BACKGROUND);

            vger.stroke_arc(c, r, 2.0, 0.0, std::f32::consts::PI, paint);

            let paint = vger.color_paint(AZURE_HIGHLIGHT);
            let a0 = lerp(*value.get(cx), THETA_MAX, THETA_MIN);
            let a1 = THETA_MAX;

            let theta = -(a0 + a1) / 2.0 + std::f32::consts::PI;
            let ap = (a0 - a1).abs() / 2.0;

            vger.stroke_arc(c, r, 2.0, theta, ap, paint);
        }),
    ))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_knob() {
        let mut cx = Context::new();

        let ui = state(|| 0.0, |s, _| knob(s));
        let sz = [100.0, 100.0].into();

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
        let s = StateHandle::<f32>::new(cx.view_id(&path), &mut cx);
        assert_eq!(*s.get(&cx), 0.0);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: [50.0, 50.0].into(),
            },
            Event::TouchMove {
                id: 0,
                position: [100.0, 50.0].into(),
                delta: [50.0, 0.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [100.0, 50.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, &mut path, &mut cx, &mut actions);
        }

        let vid = cx.view_id(&path);
        assert!(cx.state_map.contains_key(&vid));
        // State should have changed.
        assert_eq!(*s.get(&cx), 0.125);
    }
}
