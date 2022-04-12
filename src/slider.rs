use crate::*;

const SLIDER_WIDTH: f32 = 4.0;
const SLIDER_THUMB_RADIUS: f32 = 10.0;

pub struct HSlider<F> {
    value: f32,
    set_value: F,
    thumb: Color,
}

// XXX: why can't I use this?
pub trait SliderSetter: Fn(&mut Context, f32) + 'static + Copy {}

impl<F> View for HSlider<F>
where
    F: Fn(&mut Context, f32) + 'static + Copy,
{
    body_view!();
}

impl<F> HSlider<F>
where
    F: Fn(&mut Context, f32) + 'static + Copy,
{
    fn body(&self) -> impl View {
        let value = self.value;
        let thumb_color = self.thumb;
        let set_value = self.set_value.clone();
        state(
            || 0.0,
            move |width, cx| {
                let w = cx[width];
                let x = value * w;

                canvas(move |_, sz, vger| {
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
                .geom(move |cx, sz| {
                    if sz.width != w {
                        cx[width] = sz.width;
                    }
                })
                .drag(move |cx, off, _state| {
                    (set_value)(cx, (value + off.x / w).clamp(0.0, 1.0));
                })
            },
        )
        .role(accesskit::Role::Slider)
    }

    pub fn thumb_color(self, thumb_color: Color) -> Self {
        Self {
            value: self.value,
            set_value: self.set_value,
            thumb: thumb_color,
        }
    }
}

impl<B> private::Sealed for HSlider<B> {}

/// Horizontal slider built from other Views.
pub fn hslider(value: f32, set_value: impl Fn(&mut Context, f32) + 'static + Copy) -> HSlider< impl Fn(&mut Context, f32) + 'static + Copy> {
    HSlider {
        value,
        set_value,
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
        let value = self.value;
        let thumb_color = self.thumb;
        state(
            || 0.0,
            move |height, cx| {
                let h = cx[height];
                let y = value.get() * h;

                canvas(move |cx, sz, vger| {
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
                .geom(move |cx, sz| {
                    if sz.height != h {
                        cx[height] = sz.height;
                    }
                })
                .drag(move |cx, off, _state| {
                    value.with_mut(|v| *v = (*v + off.y / h).clamp(0.0, 1.0));
                })
            },
        )
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
