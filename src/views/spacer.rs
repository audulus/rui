use crate::*;

#[derive(Clone)]
pub struct Spacer {}

impl View for Spacer {
    fn draw(&self, _path: &mut IdPath, _args: &mut DrawArgs) {}
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> LocalSize {
        [0.0, 0.0].into()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl private::Sealed for Spacer {}

/// Inserts a flexible space in a stack.
pub fn spacer() -> Spacer {
    Spacer {}
}
