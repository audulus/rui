use crate::*;

/// Specifies how a region should be filled.
pub enum Paint {

    /// Fill a region with a solid color.
    Color(Color),

    /// Fill a region with a linear gradient between two colors.
    Gradient {
        start: LocalPoint,
        end: LocalPoint,
        inner_color: Color,
        outer_color: Color,
    },
}

impl Paint {
    pub fn vger_paint(&self, vger: &mut VGER) -> PaintIndex {
        match self {
            Paint::Color(color) => vger.color_paint(*color),
            Paint::Gradient { inner_color, .. } => vger.color_paint(*inner_color), // TODO
        }
    }
}
