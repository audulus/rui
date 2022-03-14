pub use crate::*;

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

pub fn button<F: Fn() + 'static>(view: impl View + 'static, f: F) -> impl View {
    view.padding(Auto)
        .background(
            rectangle()
                .corner_radius(BUTTON_CORNER_RADIUS)
                .color(BUTTON_BACKGROUND_COLOR),
        )
        .tap(f)
}
