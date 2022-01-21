
use crate::*;

pub enum Event {
    PressButton(String)
}

pub trait View {
    fn draw(&self, id: ViewID, cx: &mut Context);
    fn process(&self, event: &Event, id: ViewID, cx: &mut Context);
}

pub struct EmptyView {}

impl View for EmptyView {
    fn draw(&self, _id: ViewID, _cx: &mut Context) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) { }
}