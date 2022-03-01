use crate::*;

pub fn slider(value: impl Binding<f32>) -> impl View {
    let x = value.get();
    state(0.0, move |width| {
        let value = value.clone();
        circle()
            .offset([x, 0.0].into())
            .drag(move |off, _state| {
                value.set(value.get() + off.x);
            })
            .geom(move |sz| width.set(sz.width))
    })
}
