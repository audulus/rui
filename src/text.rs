use crate::*;

pub struct Text {
    text: String,
}

impl View for Text {
    fn draw(&self, _id: ViewID, _cx: &mut Context) {
        println!("Text({:?})", self.text);
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {}
}

impl Text {
    fn layout(&self, id: ViewID, cx: &mut Context) -> LocalSize {
        // XXX: obviously need to use vger to computer text size
        let size = LocalSize::new(self.text.len() as f32 * 10.0, 10.0);
        cx.layout.insert(id, LayoutBox{ rect: LocalRect::new(LocalPoint::zero(), size),
                                        offset: LocalOffset::zero() });
        size
    }
}

pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
    }
}
