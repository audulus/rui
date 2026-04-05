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
                canvas(move |cx, sz, vger| {
                    let c = sz.center();

                    let w = cx[width];
                    let v = value.get(cx);
                    let r = SLIDER_THUMB_RADIUS;
                    let start_x = r;
                    let end_x = w - r;
                    let x = (1.0 - v) * start_x + v * (end_x);

                    let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                    vger.fill_rect(
                        euclid::rect(
                            start_x,
                            c.y - SLIDER_WIDTH / 2.0,
                            sz.size.width - 2.0 * r,
                            SLIDER_WIDTH,
                        ),
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
                .geom(move |cx, sz, _| {
                    if sz.width != cx[width] {
                        cx[width] = sz.width;
                    }
                })
                .drag_s(value, move |v, delta, _, _| {
                    *v = (*v + delta.x / w).clamp(0.0, 1.0)
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
pub fn vslider(value: impl Binding<f32>) -> impl SliderMods {
    modview(move |opts: SliderOptions, _| {
        state(
            || 0.0,
            move |height, cx| {
                let h = cx[height];
                canvas(move |cx, sz, vger| {
                    let h = cx[height];
                    let v = value.get(cx);
                    let c = sz.center();
                    let r = SLIDER_THUMB_RADIUS;
                    let start_y = r;
                    let end_y = h - r;
                    let y = (1.0 - v) * start_y + v * end_y;

                    let paint = vger.color_paint(BUTTON_BACKGROUND_COLOR);
                    vger.fill_rect(
                        euclid::rect(
                            c.x - SLIDER_WIDTH / 2.0,
                            start_y,
                            SLIDER_WIDTH,
                            sz.height() - 2.0 * r,
                        ),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(AZURE_HIGHLIGHT_BACKGROUND);
                    vger.fill_rect(
                        euclid::rect(c.x - SLIDER_WIDTH / 2.0, start_y, SLIDER_WIDTH, y),
                        0.0,
                        paint,
                    );
                    let paint = vger.color_paint(opts.thumb);
                    vger.fill_circle([c.x, y], r, paint);
                })
                .geom(move |cx, sz, _| {
                    if sz.height != cx[height] {
                        cx[height] = sz.height;
                    }
                })
                .drag_s(value, move |v, delta, _, _| {
                    *v = (*v + delta.y / h).clamp(0.0, 1.0)
                })
            },
        )
        .role(accesskit::Role::Slider)
    })
}
