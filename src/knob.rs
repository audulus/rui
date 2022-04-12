use crate::*;

const THETA_MIN: f32 = 3.0 / 2.0 * std::f32::consts::PI;
const THETA_MAX: f32 = 7.0 / 2.0 * std::f32::consts::PI;

fn lerp(x: f32, a: f32, b: f32) -> f32 {
    (1.0 - x) * a + x * b
}

/// Knob for controlling a 0 to 1 floating point parameter.
pub fn knob(value: impl Binding<f32>) -> impl View {
    state(|| (), move |_, cx| {
        let v = *value.get(cx);

        zstack((
            circle().color(CLEAR_COLOR).drag(move |cx, off, _state| {
                *value.get_mut(cx) = (v + (off.x + off.y) / 400.0).clamp(0.0, 1.0);
            }),
            canvas(move |_, sz, vger| {
                let c = sz.center();
                let r = sz.width().min(sz.height()) / 2.0;
    
                let paint = vger.color_paint(CONTROL_BACKGROUND);
    
                vger.stroke_arc(c, r, 2.0, 0.0, std::f32::consts::PI, paint);
    
                let paint = vger.color_paint(AZURE_HIGHLIGHT);
                let a0 = lerp(v, THETA_MAX, THETA_MIN);
                let a1 = THETA_MAX;
    
                let theta = -(a0 + a1) / 2.0 + std::f32::consts::PI;
                let ap = (a0 - a1).abs() / 2.0;
    
                vger.stroke_arc(c, r, 2.0, theta, ap, paint);
            }),
        ))
    })
}
