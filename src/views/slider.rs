use crate::*;

const SLIDER_WIDTH: f32 = 4.0;
const SLIDER_THUMB_RADIUS: f32 = 10.0;

#[derive(Clone, Copy)]
pub struct SliderOptions {
    thumb: Color,
}

impl Default for SliderOptions {
    fn default() -> Self {
        Self {
            thumb: AZURE_HIGHLIGHT,
        }
    }
}

pub trait SliderMods: View + Sized {
    fn thumb_color(self, color: Color) -> Self;
}

/// Horizontal slider built from other Views.
pub fn hslider(value: impl Binding<f32>) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |width, cx| {
                let w = cx[width];
                let v = value.get(cx);
                let r = SLIDER_THUMB_RADIUS;
                let start_x = r;
                let end_x = w - r;
                let x = (1.0-v) * start_x + v * (end_x);

                canvas(move |_, sz, vger| {
                    let c = sz.center();
                    let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                    vger.fill_rect(
                        euclid::rect(start_x, c.y - SLIDER_WIDTH / 2.0, sz.size.width - 2.0*r, SLIDER_WIDTH),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(AZURE_HIGHLIGHT_BACKGROUND);
                    vger.fill_rect(
                        euclid::rect(start_x, c.y - SLIDER_WIDTH / 2.0, x, SLIDER_WIDTH),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(opts.thumb);
                    vger.fill_circle([x, c.y], r, paint);
                })
                .geom(move |cx, sz| {
                    if sz.width != w {
                        cx[width] = sz.width;
                    }
                })
                .drag(move |cx, off, _state, _key_mods, _mouse_button| {
                    value.with_mut(cx, |v| *v = (*v + off.x / w).clamp(0.0, 1.0));
                })
            },
        )
        .role(accesskit::Role::Slider)
    })
}

impl<F> SliderMods for ModView<SliderOptions, F>
where
    ModView<SliderOptions, F>: View,
{
    fn thumb_color(self, color: Color) -> Self {
        let mut opts = self.value;
        opts.thumb = color;
        ModView {
            func: self.func,
            value: opts,
        }
    }
}

/// Vertical slider built from other Views.
pub fn vslider(
    value: f32,
    set_value: impl Fn(&mut Context, f32) + 'static + Copy,
) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |height, cx| {
                let h = cx[height];
                let y = value * h;

                canvas(move |_, sz, vger| {
                    let c = sz.center();
                    let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                    vger.fill_rect(
                        euclid::rect(c.x - SLIDER_WIDTH / 2.0, 0.0, SLIDER_WIDTH, sz.height()),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(opts.thumb);
                    vger.fill_circle([c.x, y], SLIDER_THUMB_RADIUS, paint);
                })
                .geom(move |cx, sz| {
                    if sz.height != h {
                        cx[height] = sz.height;
                    }
                })
                .drag(move |cx, off, _state, _key_mods, _mouse_button| {
                    (set_value)(cx, (value + off.y / h).clamp(0.0, 1.0));
                })
            },
        )
    })
}
