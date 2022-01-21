
pub enum Event {
    PressButton(String)
}

pub trait View {
    fn draw(&self);
    fn process(&self, event: &Event);
}

pub struct EmptyView {}

impl View for EmptyView {
    fn draw(&self) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event) { }
}