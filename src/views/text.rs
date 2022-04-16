use crate::*;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: u32) -> Text;
}

/// Struct for `text`.
pub struct Text {
    text: String,
    size: u32,
}

impl Text {
    pub const DEFAULT_SIZE: u32 = 18;
}

impl View for Text {
    fn print(&self, _id: ViewId, _cx: &mut Context) {
        println!("Text({:?})", self.text);
    }
    fn process(&self, _event: &Event, _id: ViewId, _cx: &mut Context, _vger: &mut VGER) {}
    fn draw(&self, _id: ViewId, _cx: &mut Context, vger: &mut VGER) {
        let origin = vger.text_bounds(self.text.as_str(), self.size, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(self.text.as_str(), self.size, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, id: ViewId, _sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let size = vger.text_bounds(self.text.as_str(), self.size, None).size;

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), size),
                offset: LocalOffset::zero(),
            },
        );
        size
    }
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewId> {
        None
    }

    fn commands(&self, _id: ViewId, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    fn gc(&self, _id: ViewId, _cx: &mut Context, _map: &mut Vec<ViewId>) {
        // do nothing
    }

    fn access(
        &self,
        id: ViewId,
        _cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let aid = id.access_id();
        nodes.push(accesskit::Node::new(aid, accesskit::Role::LabelText));
        Some(aid)
    }
}

impl TextModifiers for Text {
    fn font_size(self, size: u32) -> Self {
        Self {
            text: self.text,
            size,
        }
    }
}

impl private::Sealed for Text {}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE,
    }
}

impl<V> View for V
where
    V: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn print(&self, _id: ViewId, _cx: &mut Context) {
        println!("Text({:?})", self);
    }
    fn process(&self, _event: &Event, _id: ViewId, _cx: &mut Context, _vger: &mut VGER) {}
    fn draw(&self, _id: ViewId, _cx: &mut Context, vger: &mut VGER) {
        let txt = &format!("{}", self);
        let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, id: ViewId, _sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let txt = &format!("{}", self);
        let size = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).size;

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), size),
                offset: LocalOffset::zero(),
            },
        );
        size
    }
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewId> {
        None
    }

    fn commands(&self, _id: ViewId, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    fn gc(&self, _id: ViewId, _cx: &mut Context, _map: &mut Vec<ViewId>) {
        // do nothing
    }

    fn access(
        &self,
        id: ViewId,
        _cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let aid = id.access_id();
        nodes.push(accesskit::Node::new(aid, accesskit::Role::LabelText));
        Some(aid)
    }
}

impl<V> TextModifiers for V
where
    V: std::fmt::Display + std::fmt::Debug + 'static,
{
    fn font_size(self, size: u32) -> Text {
        Text {
            text: format!("{}", self),
            size,
        }
    }
}

impl<V> private::Sealed for V where V: std::fmt::Display {}
