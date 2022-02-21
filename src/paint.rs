use crate::*;

pub enum Paint {
    Color(Color),
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
