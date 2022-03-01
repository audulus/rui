use crate::*;

pub fn slider(value: impl Binding<f32>) -> impl View {
    state(0.0, move |width| {
        let w = width.get();
        let x = value.get() * w;
        let value = value.clone();

        zstack! {
            rectangle()
                .color(BUTTON_BACKGROUND_COLOR);
            circle()
                .size([20.0, 20.0].into())
                .offset([x, 0.0].into())
                .drag(move |off, _state| {
                    value.set(value.get() + off.x / w);
                })
        }
        .geom(move |sz| width.set(sz.width))
    })
}
