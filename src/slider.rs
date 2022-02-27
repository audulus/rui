use crate::*;

pub fn slider(value: impl Binding<f32> + 'static) -> impl View {
    let x = value.get();
    circle().offset([x, 0.0].into()).drag(move |off, state| {
        value.set(off.x);
    })
}
