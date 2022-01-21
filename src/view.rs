
use crate::*;

pub enum Event {
    PressButton(String)
}

pub trait View {
    fn draw(&self, id: ViewID);
    fn process(&self, event: &Event, id: ViewID);
}

pub struct EmptyView {}

impl View for EmptyView {
    fn draw(&self, _id: ViewID) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event, _id: ViewID) { }
}