//! The shared looks of the dashboard: spec colors, text labels, fixed gaps, and the painted
//! surfaces (fills, borders, shadows, circles) that rui draws on a `canvas`.
//!
//! rui lays views out with y up, so a canvas paints from its bottom left corner.

use rui::vger::Color;
use rui::*;

use crate::dash::{self, Shadow};

/// Returns a spec color. vger blends in linear space and rui draws into an sRGB surface, so the
/// spec's sRGB values are converted to linear ones, which the surface turns back into them.
pub fn color(hex: u32) -> Color {
    let channel = |shift: u32| {
        let s = ((hex >> shift) & 0xff) as f32 / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    };
    Color::new(channel(16), channel(8), channel(0), 1.0)
}

/// Returns a text of `size` px in the spec color `hex`. vger has one font face, so the spec's
/// weights all render regular.
pub fn label(content: impl Into<TextStr>, size: f32, hex: u32) -> Text {
    text(content).font_size(size as u32).color(color(hex))
}

/// An empty box of a fixed size.
pub fn gap(width: f32, height: f32) -> impl View {
    spacer().size([width, height])
}

/// A view left-aligned in the width it is offered.
pub fn leading(view: impl View) -> impl View {
    hstack((view, spacer()))
}

/// A view right-aligned in the width it is offered.
pub fn trailing(view: impl View) -> impl View {
    hstack((spacer(), view))
}

/// The look of a painted surface.
#[derive(Clone, Copy)]
pub struct Surface {
    pub fill: Option<u32>,
    pub radius: f32,
    pub border: Option<u32>,
    pub shadow: Option<Shadow>,
}

impl Surface {
    /// A plain fill.
    pub const fn fill(hex: u32, radius: f32) -> Self {
        Self {
            fill: Some(hex),
            radius,
            border: None,
            shadow: None,
        }
    }

    /// The surface of every card: white, rounded, with the card shadow.
    pub const CARD: Self = Self {
        fill: Some(dash::CARD_BG),
        radius: dash::CARD_RADIUS,
        border: None,
        shadow: Some(dash::SHADOW_CARD),
    };

    pub const fn border(self, hex: u32) -> Self {
        Self {
            border: Some(hex),
            ..self
        }
    }

    pub const fn shadow(self, shadow: Shadow) -> Self {
        Self {
            shadow: Some(shadow),
            ..self
        }
    }

    /// Paints the surface over `rect`.
    pub fn paint(&self, rect: LocalRect, vger: &mut Vger) {
        if let Some(s) = self.shadow {
            paint_shadow(vger, rect, self.radius, s);
        }
        let mut inner = rect;
        if let Some(border) = self.border {
            let paint = vger.color_paint(color(border));
            vger.fill_rect(rect, self.radius, paint);
            inner = rect.inflate(-1.0, -1.0);
        }
        if let Some(fill) = self.fill {
            let paint = vger.color_paint(color(fill));
            let radius = if self.border.is_some() {
                (self.radius - 1.0).max(0.0)
            } else {
                self.radius
            };
            vger.fill_rect(inner, radius, paint);
        }
    }

    /// A view that paints the surface over the space it is offered, for `background`.
    pub fn view(self) -> impl View {
        canvas(move |_, rect, vger| self.paint(rect, vger))
    }
}

/// Paints a spec drop shadow under a rounded rectangle.
pub fn paint_shadow(vger: &mut Vger, rect: LocalRect, radius: f32, s: Shadow) {
    let paint = vger.color_paint(color(s.color).alpha(s.alpha));
    // The spec's offset points down; rui's y points up. A CSS blur radius is twice the sigma.
    let rect = rect.translate([s.x, -s.y].into());
    vger.fill_blurred_rect(rect, radius, paint, s.blur / 2.0);
}

/// Paints `content` in `size` px and `hex`, centered on `center`.
pub fn paint_centered_text(
    vger: &mut Vger,
    content: &str,
    size: u32,
    hex: u32,
    center: LocalPoint,
) {
    let bounds = vger.text_bounds(content, size, None);
    vger.save();
    vger.translate(center.to_vector() - bounds.center().to_vector());
    vger.text(content, size, color(hex), None);
    vger.restore();
}

/// A circle of `size` in `hex` with white initials of `font` px, as for the avatars.
///
/// The view keeps `initials`, so it takes an owned string, such as a cloned `TextStr`.
pub fn avatar(initials: impl Into<TextStr> + 'static, size: f32, font: u32, hex: u32) -> impl View {
    let initials = initials.into();
    canvas(move |_, rect, vger| {
        let paint = vger.color_paint(color(hex));
        vger.fill_circle(rect.center(), size / 2.0, paint);
        paint_centered_text(vger, &initials, font, 0xffffff, rect.center());
    })
    .size([size, size])
}

/// A pill: a rounded box of `height` around a text, `padding_x` on each side.
///
/// The view keeps `content`, so it takes an owned string, such as a cloned `TextStr`.
pub fn pill(
    content: impl Into<TextStr> + 'static,
    size: f32,
    hex: u32,
    height: f32,
    padding_x: f32,
    bg: u32,
) -> impl View {
    hstack((
        gap(padding_x, height),
        label(content, size, hex),
        gap(padding_x, height),
    ))
    .background(Surface::fill(bg, height / 2.0).view())
}

/// A button of `width` with a centered label.
///
/// The view keeps `content`, so it takes an owned string, such as a cloned `TextStr`.
pub fn button_box(
    content: impl Into<TextStr> + 'static,
    width: f32,
    text_hex: u32,
    surface: Surface,
) -> impl View {
    label(content, dash::FONT_SIZE, text_hex)
        .fixed_width(width)
        .fixed_height(dash::CONTROL_HEIGHT)
        .background(surface.view())
}

/// The primary button look.
pub const PRIMARY: Surface = Surface::fill(dash::PRIMARY, 8.0).shadow(dash::SHADOW_SMALL);
/// The secondary button look.
pub const SECONDARY: Surface =
    Surface::fill(dash::SECONDARY_BG, 8.0).border(dash::SECONDARY_BORDER);

/// A switch of the spec's look: rui's toggle, 40x22 with an 18 px knob. The toggle takes the
/// space it is offered, so it sits in a box of its size.
pub fn switch(on: impl Binding<bool>) -> impl View {
    Toggle::new()
        .width(dash::SWITCH_WIDTH)
        .height(dash::SWITCH_HEIGHT)
        .edge(2.0)
        .colors(
            color(dash::PRIMARY),
            color(dash::SWITCH_OFF),
            color(0xffffff),
        )
        .show(on)
        .size([dash::SWITCH_WIDTH, dash::SWITCH_HEIGHT])
}
