use crate::*;

#[derive(Clone)]
pub struct EmptyView {}

impl DynView for EmptyView {
    fn draw(&self, _path: &mut IdPath, _args: &mut DrawArgs) {}
    fn layout(&self, _path: &mut IdPath, _args: &mut LayoutArgs) -> LocalSize {
        [0.0, 0.0].into()
    }
}

impl private::Sealed for EmptyView {}
