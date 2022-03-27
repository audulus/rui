use crate::*;

pub fn toggle(state: impl Binding<bool>) -> impl View {
    let b = state.get();
    zstack((
        rectangle()
            .color(if b { AZURE_HIGHLIGHT_BACKGROUND } else { CONTROL_BACKGROUND })
            .corner_radius(10.0)
            .size([40.0, 20.0])
            .tap(move || { state.with_mut(|b| *b = !*b) }),
        circle()
            .color(if b { AZURE_HIGHLIGHT } else { MEDIUM_GRAY })
            .size([10.0, 10.0])
            .offset([if b { 25.0 } else { 5.0 }, 5.0])
    ))
}
