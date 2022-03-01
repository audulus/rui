use crate::*;

pub fn slider(value: impl Binding<f32>) -> impl View {
    let x = value.get();
    zstack! {
        rectangle(2.0).color(BUTTON_BACKGROUND_COLOR);
        state(0.0, move |width| {
            let value = value.clone();
            circle()
                .size([20.0, 20.0].into())
                .offset([x, 0.0].into())
                .drag(move |off, _state| {
                    value.set(value.get() + off.x);
                })
                .geom(move |sz| width.set(sz.width))
        })
    }
    
}
