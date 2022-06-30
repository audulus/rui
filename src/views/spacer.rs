use crate::*;

pub struct Spacer {}

impl View for Spacer {
    fn draw(&self, _id: ViewId, _cx: &mut Context, _vger: &mut Vger) {}
    fn layout(
        &self,
        _id: ViewId,
        _sz: LocalSize,
        _cx: &mut Context,
        _vger: &mut Vger,
    ) -> LocalSize {
        [0.0, 0.0].into()
    }

    fn is_flexible(&self) -> bool {
        true
    }
}

impl private::Sealed for Spacer {}

pub fn spacer() -> Spacer {
    Spacer {}
}
