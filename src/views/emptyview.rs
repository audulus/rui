use crate::*;

pub struct EmptyView {}

impl View for EmptyView {
    fn print(&self, _id: ViewId, _cx: &mut Context) {
        println!("EmptyView");
    }
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
}

impl private::Sealed for EmptyView {}
