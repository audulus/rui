use crate::*;

/// Toggle switch.
pub fn toggle(on: bool, set: impl Fn(&mut Context, bool) + 'static) -> impl View {
    zstack((
        rectangle()
            .color(if on {
                AZURE_HIGHLIGHT_BACKGROUND
            } else {
                CONTROL_BACKGROUND
            })
            .corner_radius(10.0)
            .size([40.0, 20.0])
            .tap(move |cx| set(cx, !on)),
        circle()
            .color(if on { AZURE_HIGHLIGHT } else { MEDIUM_GRAY })
            .size([10.0, 10.0])
            .offset([if on { 25.0 } else { 5.0 }, 5.0]),
    ))
}
