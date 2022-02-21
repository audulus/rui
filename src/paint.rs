use crate::*;

enum Paint {
    Color(Color),
    Gradient {
        start: LocalPoint,
        end: LocalPoint,
        inner_color: Color,
        outer_color: Color,
    },
}
