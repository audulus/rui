
use crate::*;

pub struct Text {
    text: String
}

impl View for Text {
    fn draw(&self, _id: ViewID, _cx: &mut Context) {
        println!("Text({:?})", self.text);
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {}
}

pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name)
    }
}