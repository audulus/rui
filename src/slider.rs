use crate::*;

const SLIDER_WIDTH :f32 = 4.0;
const SLIDER_THUMB_RADIUS : f32 = 10.0;

/// Horizontal slider built from other Views.
pub fn hslider(value: impl Binding<f32>) -> impl View {
    state(0.0, move |width| {
        let w = width.get();
        let x = value.get() * w;
        let value = value.clone();

        zstack((
            rectangle()
                .color(CLEAR_COLOR)
                .drag(move |off, _state| {
                    value.set( (value.get() + off.x / w).clamp(0.0,1.0));
                }),
            canvas(move |sz, vger| {
                let c = sz.center();
                let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                vger.fill_rect(
                    [0.0, c.y-SLIDER_WIDTH/2.0].into(),
                    [sz.width(), c.y+SLIDER_WIDTH/2.0].into(),
                    0.0,
                    paint
                );
                let paint = vger.color_paint(AZURE_HIGHLIGHT);
                vger.fill_circle(
                    [x, c.y],
                    SLIDER_THUMB_RADIUS,
                    paint
                );
            }),
        ))
        .geom(move |sz| if sz.width != w { width.set(sz.width) })
    })
}

struct HSlider<V: View, B: Binding<f32>> {
    body: V,
    value: B,
    thumb_color: Color
}

impl<V, B> View for HSlider<V, B> where V:View, B:Binding<f32> {
    modifier_view!();
}

impl<V, B> HSlider<V, B> where V:View, B:Binding<f32> {
    fn thunb_color(self, thumb_color: Color) -> Self {
        Self {
            body: self.body,
            value: self.value,
            thumb_color
        }
    }
}

/// Vertical slider built from other Views.
pub fn vslider(value: impl Binding<f32>) -> impl View {
    state(0.0, move |height| {
        let h = height.get();
        let y = value.get() * h;
        let value = value.clone();

        zstack((
            rectangle()
                .color(CLEAR_COLOR)
                .drag(move |off, _state| {
                    value.set( (value.get() + off.y / h).clamp(0.0,1.0));
                }),
            canvas(move |sz, vger| {
                let c = sz.center();
                let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                vger.fill_rect(
                    [c.x-SLIDER_WIDTH/2.0, 0.0].into(),
                    [c.x+SLIDER_WIDTH/2.0, sz.height()].into(),
                    0.0,
                    paint
                );
                let paint = vger.color_paint(AZURE_HIGHLIGHT);
                vger.fill_circle(
                    [c.x, y],
                    SLIDER_THUMB_RADIUS,
                    paint
                );
            }),
        ))
        .geom(move |sz| if sz.height != h { height.set(sz.width) })
    })
}
