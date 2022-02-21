use crate::*;

pub struct Text {
    text: String,
}

impl Text {
    pub const DEFAULT_SIZE: u32 = 18;
}

impl View for Text {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("Text({:?})", self.text);
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context) {}
    fn draw(&self, _id: ViewID, _cx: &mut Context, vger: &mut VGER) {
        let origin = vger
            .text_bounds(self.text.as_str(), Button::DEFAULT_SIZE, None)
            .origin;

        vger.save();
        vger.translate([-origin.x, -origin.y].into());
        vger.text(self.text.as_str(), Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, id: ViewID, _sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let size = vger
            .text_bounds(self.text.as_str(), Button::DEFAULT_SIZE, None)
            .size;

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), size),
                offset: LocalOffset::zero(),
            },
        );
        size
    }
    fn hittest(&self, id: ViewID, pt: LocalPoint, cx: &mut Context, vger: &mut VGER) -> Option<ViewID> {
        None
    }
}

impl Text {
    modifiers!();
}

pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
    }
}
