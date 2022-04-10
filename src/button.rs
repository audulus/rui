use crate::*;
use accesskit::Role;

pub const BUTTON_CORNER_RADIUS: f32 = 5.0;

/// Calls a function when the button is tapped.
pub fn button<F: Fn() + 'static>(view: impl View + 'static, f: F) -> impl View {
    view.padding(Auto)
        .background(
            rectangle()
                .corner_radius(BUTTON_CORNER_RADIUS)
                .color(BUTTON_BACKGROUND_COLOR),
        )
        .tap(f)
        .role(Role::Button)
}
