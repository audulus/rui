pub use crate::*;

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

pub fn button<F: Fn() + 'static>(name: &str, f: F) -> impl View {
    text(&name)
        .padding(Auto)
        .background(
            rectangle()
                .corner_radius(BUTTON_CORNER_RADIUS)
                .color(BUTTON_BACKGROUND_COLOR),
        )
        .tap(f)
}
