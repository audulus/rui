use crate::*;

pub enum Event {
    PressButton(String),
}

pub trait View {
    fn print(&self, id: ViewID, cx: &mut Context);
    fn process(&self, event: &Event, id: ViewID, cx: &mut Context);
    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER);
    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context) -> LocalSize;
}

pub struct EmptyView {}

impl View for EmptyView {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {}
    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {}
    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context) -> LocalSize {
        [0.0, 0.0].into()
    }
}
