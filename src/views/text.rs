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
    fn draw(&self, _id: ViewId, args: &mut DrawArgs) {
        let vger = &mut args.vger;
        let origin = vger.text_bounds(self.text.as_str(), self.size, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(self.text.as_str(), self.size, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, _id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        args.vger.text_bounds(self.text.as_str(), self.size, None).size
    }
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        None
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
    fn draw(&self, _id: ViewId, args: &mut DrawArgs) {
        let txt = &format!("{}", self);
        let vger = &mut args.vger;
        let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let txt = &format!("{}", self);
        let size = args.vger.text_bounds(txt, Text::DEFAULT_SIZE, None).size;

        args.cx.layout.insert(
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
        _vger: &mut Vger,
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
