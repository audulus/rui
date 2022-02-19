use crate::*;

#[derive(Clone, Debug)]
pub enum EventKind {
    PressButton(String),
    TouchBegin{id:usize},
    TouchEnd{id:usize},
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub position: LocalPoint
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
