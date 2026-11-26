use crate::*;
use std::sync::Arc;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: u32) -> Text;
    fn color(self, color: Color) -> Text;
    fn max_width(self, max_width: f32) -> Text;
    /// Wraps the text to the width it is offered.
    fn wrap(self) -> Text;
}

/// The string a [`Text`] shows: a literal, or a shared string that clones without copying.
///
/// Literals are kept as `&'static str`. Any other string is copied once into an `Arc<str>`, so
/// a view rebuilt every frame from strings made once (held as `Arc<str>`) doesn't allocate.
///
/// Two `TextStr`s are equal, ordered and hashed by their text, whichever way they hold it.
#[derive(Clone, Debug)]
pub enum TextStr {
    Static(&'static str),
    Shared(Arc<str>),
}

impl PartialEq for TextStr {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl Eq for TextStr {}

impl PartialOrd for TextStr {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextStr {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (**self).cmp(&**other)
    }
}

impl std::hash::Hash for TextStr {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl std::borrow::Borrow<str> for TextStr {
    fn borrow(&self) -> &str {
        self
    }
}

impl std::ops::Deref for TextStr {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            TextStr::Static(s) => s,
            TextStr::Shared(s) => s,
        }
    }
}

impl std::fmt::Display for TextStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl From<&'static str> for TextStr {
    fn from(s: &'static str) -> Self {
        TextStr::Static(s)
    }
}

impl From<Arc<str>> for TextStr {
    fn from(s: Arc<str>) -> Self {
        TextStr::Shared(s)
    }
}

impl From<&Arc<str>> for TextStr {
    fn from(s: &Arc<str>) -> Self {
        TextStr::Shared(s.clone())
    }
}

impl From<String> for TextStr {
    fn from(s: String) -> Self {
        TextStr::Shared(s.into())
    }
}

/// Copies the string. Hold an `Arc<str>` instead to show it without copying.
impl From<&String> for TextStr {
    fn from(s: &String) -> Self {
        TextStr::Shared(s.as_str().into())
    }
}

impl From<&TextStr> for TextStr {
    fn from(s: &TextStr) -> Self {
        s.clone()
    }
}

/// Struct for `text`.
#[derive(Clone)]
pub struct Text {
    text: TextStr,
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
    fn break_width(&self, vid: ViewId, cx: &mut Context) -> Option<f32> {
        if self.wrap {
            Some(cx.get_layout(vid).rect.width())
        } else {
            self.max_width
        }
    }
}

impl DynView for Text {
    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let max_width = self.break_width(vid, args.cx);
        let vger = &mut args.vger;
        let origin = vger.text_bounds(&self.text, self.size, max_width).origin;

        vger.save();
        vger.translate([-origin.x, -origin.y]);
        vger.text(&self.text, self.size, self.color, max_width);
        vger.restore();
    }
    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        if self.wrap {
            let width = args.sz.width.max(1.0);
            let size = (args.text_bounds)(&self.text, self.size, Some(width)).size;
            args.cx.update_layout(
                vid,
                LayoutBox {
                    rect: LocalRect::new(LocalPoint::zero(), [width, size.height].into()),
                    offset: LocalOffset::zero(),
                },
            );
            [width, size.height].into()
        } else {
            (args.text_bounds)(&self.text, self.size, self.max_width).size
        }
    }
    fn hittest(&self, _vid: ViewId, _pt: LocalPoint, _cx: &mut Context) -> Option<ViewId> {
        None
    }

    fn access(
        &self,
        vid: ViewId,
        _cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let aid = vid.access_id();
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
        builder.set_name(&*self.text);
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
///
/// Takes a literal, a `String`, or an `Arc<str>` (see [`TextStr`]). A borrowed `&str` that
/// isn't `'static` has to be owned first: pass `s.to_owned()`, or keep the string as an
/// `Arc<str>` to show it without copying each time the view is built.
pub fn text(name: impl Into<TextStr>) -> Text {
    Text {
        text: name.into(),
        size: Text::DEFAULT_SIZE,
        color: TEXT_COLOR,
        max_width: None,
        wrap: false,
    }
}

macro_rules! impl_text {
    ( $ty:ident ) => {
        impl DynView for $ty {
            fn draw(&self, _vid: ViewId, args: &mut DrawArgs) {
                let txt = &format!("{}", self);
                let vger = &mut args.vger;
                let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

                vger.save();
                vger.translate([-origin.x, -origin.y]);
                vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
                vger.restore();
            }
            fn layout(&self, _vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
                let txt = &format!("{}", self);
                (args.text_bounds)(txt, Text::DEFAULT_SIZE, None).size
            }

            fn access(
                &self,
                vid: ViewId,
                _cx: &mut Context,
                nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
            ) -> Option<accesskit::NodeId> {
                let aid = vid.access_id();
                let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
                builder.set_name(format!("{}", self));
                nodes.push((aid, builder.build()));
                Some(aid)
            }
        }

        impl TextModifiers for $ty {
            fn font_size(self, size: u32) -> Text {
                Text {
                    text: format!("{}", self).into(),
                    size,
                    color: TEXT_COLOR,
                    max_width: None,
                    wrap: false,
                }
            }
            fn color(self, color: Color) -> Text {
                Text {
                    text: format!("{}", self).into(),
                    size: Text::DEFAULT_SIZE,
                    color,
                    max_width: None,
                    wrap: false,
                }
            }

            fn max_width(self, max_width: f32) -> Text {
                Text {
                    text: format!("{}", self).into(),
                    size: Text::DEFAULT_SIZE,
                    color: TEXT_COLOR,
                    max_width: Some(max_width),
                    wrap: false,
                }
            }

            fn wrap(self) -> Text {
                text(format!("{}", self)).wrap()
            }
        }
    };
}

// XXX: this used to be generic for any Display but
//      that was causing trouble with adding Clone to view.
//      Perhaps a rust wizard can figure out why.
impl_text!(u32);
impl_text!(i32);
impl_text!(u64);
impl_text!(i64);
impl_text!(f32);
impl_text!(f64);

/// Draws a string as a text of the default size and color.
macro_rules! impl_str_text {
    ( $($ty:ty),* ) => { $(
        impl DynView for $ty {
            fn draw(&self, _vid: ViewId, args: &mut DrawArgs) {
                let txt: &str = &**self;
                let vger = &mut args.vger;
                let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

                vger.save();
                vger.translate([-origin.x, -origin.y]);
                vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
                vger.restore();
            }
            fn layout(&self, _vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
                (args.text_bounds)(&**self, Text::DEFAULT_SIZE, None).size
            }

            fn access(
                &self,
                vid: ViewId,
                _cx: &mut Context,
                nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
            ) -> Option<accesskit::NodeId> {
                let aid = vid.access_id();
                let mut builder = accesskit::NodeBuilder::new(accesskit::Role::Label);
                builder.set_name(&**self);
                nodes.push((aid, builder.build()));
                Some(aid)
            }
        }

        impl TextModifiers for $ty {
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
    )* };
}

impl_str_text!(&'static str, String, Arc<str>, TextStr);

impl<V> private::Sealed for V where V: std::fmt::Display {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_str_does_not_copy() {
        assert!(matches!(
            TextStr::from("literal"),
            TextStr::Static("literal")
        ));

        let shared: Arc<str> = Arc::from("shared");
        let TextStr::Shared(s) = TextStr::from(&shared) else {
            panic!("an Arc<str> should stay shared");
        };
        assert!(Arc::ptr_eq(&s, &shared));

        assert_eq!(&*text(String::from("owned")).text, "owned");
    }

    #[test]
    fn text_str_compares_by_text() {
        use std::collections::HashSet;

        let literal = TextStr::from("Save");
        let owned = TextStr::from(String::from("Save"));
        assert_eq!(literal, owned);
        assert!(TextStr::from("a") < TextStr::from(String::from("b")));

        let set: HashSet<TextStr> = [literal].into_iter().collect();
        assert!(set.contains(&owned));
        assert!(set.contains("Save"));
    }
}
