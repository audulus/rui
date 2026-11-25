use crate::*;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: u32) -> Text;
    fn color(self, color: Color) -> Text;
    fn max_width(self, max_width: f32) -> Text;
    /// Wraps the text to the width it is offered.
    fn wrap(self) -> Text;
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: String,
    size: u32,
    color: Color,
    max_width: Option<f32>,
    wrap: bool,
}

impl Text {
    pub const DEFAULT_SIZE: u32 = 18;
    pub fn color(self, color: Color) -> Text {
        Text { color, ..self }
    }
}

impl Text {
    /// The width the text breaks its lines at, if any.
    fn break_width(&self, path: &IdPath, cx: &mut Context) -> Option<f32> {
        if self.wrap {
            Some(cx.get_layout(path).rect.width())
        } else {
            self.max_width
        }
    }
}

impl DynView for Text {
    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let max_width = self.break_width(path, args.cx);
        let vger = &mut args.vger;
        let origin = vger
            .text_bounds(self.text.as_str(), self.size, max_width)
            .origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(self.text.as_str(), self.size, self.color, max_width);
        vger.restore();
    }
    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        if self.wrap {
            let width = args.sz.width.max(1.0);
            let size = (args.text_bounds)(self.text.as_str(), self.size, Some(width)).size;
            args.cx.update_layout(
                path,
                LayoutBox {
                    rect: LocalRect::new(LocalPoint::zero(), [width, size.height].into()),
                    offset: LocalOffset::zero(),
                },
            );
            [width, size.height].into()
        } else {
            (args.text_bounds)(self.text.as_str(), self.size, self.max_width).size
        }
    }
    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        if self.wrap {
            map.push(cx.view_id(path));
        }
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
        Self { size, ..self }
    }
    fn color(self, color: Color) -> Text {
        Text { color, ..self }
    }
    fn max_width(self, max_width: f32) -> Text {
        Text {
            max_width: Some(max_width),
            ..self
        }
    }
    fn wrap(self) -> Text {
        Text { wrap: true, ..self }
    }
}

impl private::Sealed for Text {}

/// Shows a string as a label (not editable).
pub fn text(name: &str) -> Text {
    Text {
        text: String::from(name),
        size: Text::DEFAULT_SIZE,
        color: TEXT_COLOR,
        max_width: None,
        wrap: false,
    }
}

macro_rules! impl_text {
    ( $ty:ident ) => {
        impl DynView for $ty {
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

        impl TextModifiers for $ty {
            fn font_size(self, size: u32) -> Text {
                Text {
                    text: format!("{}", self),
                    size,
                    color: TEXT_COLOR,
                    max_width: None,
                    wrap: false,
                }
            }
            fn color(self, color: Color) -> Text {
                Text {
                    text: format!("{}", self),
                    size: Text::DEFAULT_SIZE,
                    color,
                    max_width: None,
                    wrap: false,
                }
            }

            fn max_width(self, max_width: f32) -> Text {
                Text {
                    text: format!("{}", self),
                    size: Text::DEFAULT_SIZE,
                    color: TEXT_COLOR,
                    max_width: Some(max_width),
                    wrap: false,
                }
            }

            fn wrap(self) -> Text {
                text(&format!("{}", self)).wrap()
            }
        }
    };
}

// XXX: this used to be generic for any Display but
//      that was causing trouble with adding Clone to view.
//      Perhaps a rust wizard can figure out why.
impl_text!(String);
impl_text!(u32);
impl_text!(i32);
impl_text!(u64);
impl_text!(i64);
impl_text!(f32);
impl_text!(f64);

// XXX: Can't do impl_text!(&'static str)
impl DynView for &'static str {
    fn draw(&self, _path: &mut IdPath, args: &mut DrawArgs) {
        let txt = &self.to_string();
        let vger = &mut args.vger;
        let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
        vger.restore();
    }
    fn layout(&self, _path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let txt = &self.to_string();
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
        builder.set_name(self.to_string());
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl TextModifiers for &'static str {
    fn font_size(self, size: u32) -> Text {
        text(self).font_size(size)
    }
    fn color(self, color: Color) -> Text {
        text(self).color(color)
    }
    fn max_width(self, max_width: f32) -> Text {
        text(self).max_width(max_width)
    }
    fn wrap(self) -> Text {
        text(self).wrap()
    }
}

impl<V> private::Sealed for V where V: std::fmt::Display {}
