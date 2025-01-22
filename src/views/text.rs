use crate::*;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: u32) -> Text;
    fn color(self, color: Color) -> Text;
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: String,
    size: u32,
    color: Color,
}

impl Text {
    pub const DEFAULT_SIZE: u32 = 18;
    pub fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
        }
    }
}

impl DynView for Text {
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let vger = &mut args.vger;
        let origin = vger.text_bounds(self.text.as_str(), self.size, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(self.text.as_str(), self.size, self.color, None);
        vger.restore();
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        (args.text_bounds)(self.text.as_str(), self.size, None).size
    }
    fn hittest(&self, _path: &mut IdPath, _pt: LocalPoint, _cx: &mut Context) -> Option<ViewId> {
        None
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
        builder.set_name(self.text.clone());
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl TextModifiers for Text {
    fn font_size(self, size: u32) -> Self {
        Self {
            text: self.text,
            color: self.color,
            size,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: self.text,
            size: self.size,
            color,
        }
    }
}

impl private::Sealed for Text {}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE,
        color: TEXT_COLOR,
    }
}

impl DynView for String {
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let txt = &format!("{}", self);
        let vger = &mut args.vger;
        let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let txt = &format!("{}", self);
        (args.text_bounds)(txt, Text::DEFAULT_SIZE, None).size
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
        builder.set_name(format!("{}", self));
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl DynView for &'static str {
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let txt = &format!("{}", self);
        let vger = &mut args.vger;
        let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let txt = &format!("{}", self);
        (args.text_bounds)(txt, Text::DEFAULT_SIZE, None).size
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = cx.view_id(path).access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
        builder.set_name(format!("{}", self));
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl TextModifiers for String
{
    fn font_size(self, size: u32) -> Text {
        Text {
            text: format!("{}", self),
            size,
            color: TEXT_COLOR,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color,
        }
    }
}

impl TextModifiers for &'static str
{
    fn font_size(self, size: u32) -> Text {
        Text {
            text: format!("{}", self),
            size,
            color: TEXT_COLOR,
        }
    }
    fn color(self, color: Color) -> Text {
        Text {
            text: format!("{}", self),
            size: Text::DEFAULT_SIZE,
            color,
        }
    }
}

impl<V> private::Sealed for V where V: std::fmt::Display {}
