use crate::*;

const SLIDER_WIDTH :f32 = 4.0;

/// Horizontal slider built from other Views.
pub fn slider(value: impl Binding<f32>) -> impl View {
    state(0.0, move |width| {
        let w = width.get();
        let x = value.get() * w;
        let value = value.clone();

        zstack((
            canvas(|sz, vger| {
                let c = sz.center();
                let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                vger.fill_rect(
                    [0.0, c.y-SLIDER_WIDTH/2.0].into(),
                    [sz.width(), c.y+SLIDER_WIDTH/2.0].into(),
                    0.0,
                    paint
                )
            }),
            circle()
                .size([20.0, 20.0])
                .offset([x, 0.0])
                .drag(move |off, _state| {
                    value.set( (value.get() + off.x / w).clamp(0.0,1.0));
                }),
        ))
        .geom(move |sz| width.set(sz.width))
    })
}
