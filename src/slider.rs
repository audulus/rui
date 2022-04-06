use crate::*;

const SLIDER_WIDTH: f32 = 4.0;
const SLIDER_THUMB_RADIUS: f32 = 10.0;

pub struct HSlider<B> {
    value: B,
    thumb: Color,
}

impl<B> View for HSlider<B>
where
    B: Binding<f32>,
{
    body_view!();
}

impl<B> HSlider<B>
where
    B: Binding<f32>,
{
    fn body(&self) -> impl View {
        let value = self.value.clone();
        let thumb_color = self.thumb;
        state(0.0, move |width| {
            let w = width.get();
            let x = value.get() * w;
            let value = value.clone();

            canvas(move |sz, vger| {
                let c = sz.center();
                let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                vger.fill_rect(
                    euclid::rect(0.0, c.y - SLIDER_WIDTH / 2.0, sz.width(), SLIDER_WIDTH),
                    0.0,
                    paint,
                );
                let paint = vger.color_paint(AZURE_HIGHLIGHT_BACKGROUND);
                vger.fill_rect(
                    euclid::rect(0.0, c.y - SLIDER_WIDTH / 2.0, x, SLIDER_WIDTH),
                    0.0,
                    paint,
                );
                let paint = vger.color_paint(thumb_color);
                vger.fill_circle([x, c.y], SLIDER_THUMB_RADIUS, paint);
            })
            .geom(move |sz| {
                if sz.width != w {
                    width.set(sz.width)
                }
            })
            .drag(move |off, _state| {
                value.with_mut(|v| *v = (*v + off.x / w).clamp(0.0, 1.0));
            })
        })
        .role(accesskit::Role::Slider)
    }

    pub fn thumb_color(self, thumb_color: Color) -> Self {
        Self {
            value: self.value,
            thumb: thumb_color,
        }
    }
}

impl<B> private::Sealed for HSlider<B> {}

/// Horizontal slider built from other Views.
pub fn hslider(value: impl Binding<f32>) -> HSlider<impl Binding<f32>> {
    HSlider {
        value,
        thumb: AZURE_HIGHLIGHT,
    }
}

pub struct VSlider<B> {
    value: B,
    thumb: Color,
}

impl<B> View for VSlider<B>
where
    B: Binding<f32>,
{
    body_view!();
}

impl<B> VSlider<B>
where
    B: Binding<f32>,
{
    fn body(&self) -> impl View {
        let value = self.value.clone();
        let thumb_color = self.thumb;
        state(0.0, move |height| {
            let h = height.get();
            let y = value.get() * h;
            let value = value.clone();

            canvas(move |sz, vger| {
                let c = sz.center();
                let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                vger.fill_rect(
                    euclid::rect(c.x - SLIDER_WIDTH / 2.0, 0.0, SLIDER_WIDTH, sz.height()),
                    0.0,
                    paint,
                );
                let paint = vger.color_paint(thumb_color);
                vger.fill_circle([c.x, y], SLIDER_THUMB_RADIUS, paint);
            })
            .geom(move |sz| {
                if sz.height != h {
                    height.set(sz.width)
                }
            })
            .drag(move |off, _state| {
                value.with_mut(|v| *v = (*v + off.y / h).clamp(0.0, 1.0));
            })
        })
    }

    pub fn thumb_color(self, thumb_color: Color) -> Self {
        Self {
            value: self.value,
            thumb: thumb_color,
        }
    }
}

impl<B> private::Sealed for VSlider<B> {}

/// Horizontal slider built from other Views.
pub fn vslider(value: impl Binding<f32>) -> VSlider<impl Binding<f32>> {
    VSlider {
        value,
        thumb: AZURE_HIGHLIGHT,
    }
}
