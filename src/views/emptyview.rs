use crate::*;

pub struct EmptyView {}

impl View for EmptyView {
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

pub struct EmptyView2<Data> {
    phantom: std::marker::PhantomData<fn() -> Data>,
}

impl<Data> View2<Data> for EmptyView2<Data> {
    type State = ();

    fn draw(
        &self,
        _id: ViewId,
        _cx: &mut Context,
        _vger: &mut Vger,
        _state: &mut Self::State,
        _data: &Data,
    ) {
    }

    fn layout(
        &self,
        _id: ViewId,
        _sz: LocalSize,
        _cx: &mut Context,
        _vger: &mut Vger,
        _state: &mut Self::State,
        _data: &Data,
    ) -> LocalSize {
        [0.0, 0.0].into()
    }
}

pub fn empty_view2<Data>() -> impl View2<Data> {
    EmptyView2 {
        phantom: Default::default(),
    }
}
