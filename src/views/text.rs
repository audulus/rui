use crate::*;
use std::sync::Arc;

pub trait TextModifiers: View + Sized {
    fn font_size(self, size: u32) -> Text;
    fn color(self, color: Color) -> Text;
    fn max_width(self, max_width: f32) -> Text;
    /// Wraps the text to the width it is offered.
    fn wrap(self) -> Text;
}

/// The string a [`Text`] shows, held so that showing it again doesn't allocate.
///
/// - A literal is kept as its `&'static str`.
/// - A string of up to [`TextStr::INLINE`] bytes is copied into the `TextStr` itself, so it
///   needs no allocation at all and clones by copying 24 bytes.
/// - A longer one is copied once into an `Arc<str>`, which clones by bumping a count. An
///   `Arc<str>` passed in is kept as it is, so strings made once and held as `Arc<str>` are
///   shared, whatever their length.
///
/// `format_args!` converts too, writing short results straight into the `TextStr`:
/// `text(format_args!("{count} items"))` doesn't allocate while the text is short.
///
/// Two `TextStr`s are equal, ordered and hashed by their text, however they hold it.
#[derive(Clone)]
pub struct TextStr(Repr);

#[derive(Clone)]
enum Repr {
    Static(&'static str),
    /// The first `len` bytes of `buf`, always valid UTF-8.
    Inline {
        len: u8,
        buf: [u8; TextStr::INLINE],
    },
    Shared(Arc<str>),
}

impl TextStr {
    /// The longest string, in bytes, kept inline. It makes a `TextStr` as big as a `String`.
    pub const INLINE: usize = 22;

    /// Copies `s`: inline when it's short, into an `Arc<str>` otherwise. Use it for a borrowed
    /// `&str` that isn't `'static`.
    pub fn new(s: &str) -> Self {
        Self::inline(s).unwrap_or_else(|| TextStr(Repr::Shared(s.into())))
    }

    fn inline(s: &str) -> Option<Self> {
        (s.len() <= Self::INLINE).then(|| {
            let mut buf = [0; Self::INLINE];
            buf[..s.len()].copy_from_slice(s.as_bytes());
            TextStr(Repr::Inline {
                len: s.len() as u8,
                buf,
            })
        })
    }

    /// Whether the text is kept inline, without an allocation.
    pub fn is_inline(&self) -> bool {
        matches!(self.0, Repr::Inline { .. })
    }
}

impl std::ops::Deref for TextStr {
    type Target = str;
    fn deref(&self) -> &str {
        match &self.0 {
            Repr::Static(s) => s,
            Repr::Inline { len, buf } => {
                std::str::from_utf8(&buf[..*len as usize]).expect("inline text is UTF-8")
            }
            Repr::Shared(s) => s,
        }
    }
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

impl std::fmt::Display for TextStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self)
    }
}

impl std::fmt::Debug for TextStr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl From<&'static str> for TextStr {
    fn from(s: &'static str) -> Self {
        TextStr(Repr::Static(s))
    }
}

impl From<Arc<str>> for TextStr {
    fn from(s: Arc<str>) -> Self {
        TextStr(Repr::Shared(s))
    }
}

impl From<&Arc<str>> for TextStr {
    fn from(s: &Arc<str>) -> Self {
        TextStr(Repr::Shared(s.clone()))
    }
}

/// Inline when it's short, otherwise copied once into an `Arc<str>`.
impl From<String> for TextStr {
    fn from(s: String) -> Self {
        TextStr::inline(&s).unwrap_or_else(|| TextStr(Repr::Shared(s.into())))
    }
}

/// Inline when it's short, otherwise copied into an `Arc<str>`. Hold an `Arc<str>` instead to
/// show a long string without copying it.
impl From<&String> for TextStr {
    fn from(s: &String) -> Self {
        TextStr::new(s)
    }
}

impl From<&TextStr> for TextStr {
    fn from(s: &TextStr) -> Self {
        s.clone()
    }
}

/// Writes the text inline while it fits, then spills into a `String`.
impl From<std::fmt::Arguments<'_>> for TextStr {
    fn from(args: std::fmt::Arguments<'_>) -> Self {
        if let Some(s) = args.as_str() {
            return TextStr::from(s);
        }
        let mut w = InlineWriter {
            len: 0,
            buf: [0; TextStr::INLINE],
            spill: None,
        };
        std::fmt::Write::write_fmt(&mut w, args).expect("formatting a text failed");
        match w.spill {
            Some(s) => TextStr(Repr::Shared(s.into())),
            None => TextStr(Repr::Inline {
                len: w.len as u8,
                buf: w.buf,
            }),
        }
    }
}

/// A writer that fills an inline buffer, moving to a `String` once the text outgrows it.
struct InlineWriter {
    len: usize,
    buf: [u8; TextStr::INLINE],
    spill: Option<String>,
}

impl std::fmt::Write for InlineWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        if let Some(spill) = &mut self.spill {
            spill.push_str(s);
        } else if self.len + s.len() <= TextStr::INLINE {
            self.buf[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
            self.len += s.len();
        } else {
            // Only whole `str`s are written, so the bytes so far are valid UTF-8.
            let so_far = std::str::from_utf8(&self.buf[..self.len]).map_err(|_| std::fmt::Error)?;
            let mut spill = String::with_capacity(self.len + s.len());
            spill.push_str(so_far);
            spill.push_str(s);
            self.spill = Some(spill);
        }
        Ok(())
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
/// Takes a literal, a `String`, an `Arc<str>` or `format_args!(…)` (see [`TextStr`]). A
/// borrowed `&str` that isn't `'static` goes through [`TextStr::new`], which copies it; keep a
/// long string as an `Arc<str>` to show it without copying each time the view is built.
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
                let txt = &*TextStr::from(format_args!("{}", self));
                let vger = &mut args.vger;
                let origin = vger.text_bounds(txt, Text::DEFAULT_SIZE, None).origin;

                vger.save();
                vger.translate([-origin.x, -origin.y]);
                vger.text(txt, Text::DEFAULT_SIZE, TEXT_COLOR, None);
                vger.restore();
            }
            fn layout(&self, _vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
                let txt = &*TextStr::from(format_args!("{}", self));
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
                builder.set_name(&*TextStr::from(format_args!("{}", self)));
                nodes.push((aid, builder.build()));
                Some(aid)
            }
        }

        impl TextModifiers for $ty {
            fn font_size(self, size: u32) -> Text {
                Text {
                    text: format_args!("{}", self).into(),
                    size,
                    color: TEXT_COLOR,
                    max_width: None,
                    wrap: false,
                }
            }
            fn color(self, color: Color) -> Text {
                Text {
                    text: format_args!("{}", self).into(),
                    size: Text::DEFAULT_SIZE,
                    color,
                    max_width: None,
                    wrap: false,
                }
            }

            fn max_width(self, max_width: f32) -> Text {
                Text {
                    text: format_args!("{}", self).into(),
                    size: Text::DEFAULT_SIZE,
                    color: TEXT_COLOR,
                    max_width: Some(max_width),
                    wrap: false,
                }
            }

            fn wrap(self) -> Text {
                text(format_args!("{}", self)).wrap()
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
            TextStr::from("literal").0,
            Repr::Static("literal")
        ));

        let shared: Arc<str> = Arc::from("shared");
        let Repr::Shared(s) = TextStr::from(&shared).0 else {
            panic!("an Arc<str> should stay shared");
        };
        assert!(Arc::ptr_eq(&s, &shared));

        assert_eq!(&*text(String::from("owned")).text, "owned");
    }

    #[test]
    fn text_str_keeps_short_strings_inline() {
        assert_eq!(
            std::mem::size_of::<TextStr>(),
            std::mem::size_of::<String>()
        );

        let short = "a".repeat(TextStr::INLINE);
        let long = "a".repeat(TextStr::INLINE + 1);
        assert!(TextStr::new(&short).is_inline());
        assert!(!TextStr::new(&long).is_inline());
        assert!(TextStr::from(short.clone()).is_inline());
        assert!(!TextStr::from(long.clone()).is_inline());
        assert_eq!(&*TextStr::new(&short), short);
        assert_eq!(&*TextStr::from(long.clone()), long);

        // Multi-byte characters near the limit.
        let accents = "é".repeat(TextStr::INLINE / 2);
        assert!(TextStr::new(&accents).is_inline());
        assert_eq!(&*TextStr::new(&accents), accents);
    }

    #[test]
    fn text_str_formats_inline() {
        let n = 1042;
        let short = TextStr::from(format_args!("#{n} ${:.2}", 3.5));
        assert!(short.is_inline());
        assert_eq!(&*short, "#1042 $3.50");

        // Spills into a shared string once the text outgrows the buffer, in the middle of
        // one argument or across several.
        let long = TextStr::from(format_args!("{}", "x".repeat(40)));
        assert!(!long.is_inline());
        assert_eq!(&*long, "x".repeat(40));
        let parts = TextStr::from(format_args!("{n}{n}{n}{n}{n}{n}{n}"));
        assert_eq!(&*parts, "1042".repeat(7));

        // A literal stays static.
        assert!(matches!(
            TextStr::from(format_args!("plain")).0,
            Repr::Static("plain")
        ));
    }

    #[test]
    fn text_str_compares_by_text() {
        use std::collections::HashSet;

        let literal = TextStr::from("Save");
        let inline = TextStr::from(String::from("Save"));
        let shared = TextStr::from(Arc::<str>::from("Save"));
        assert_eq!(literal, inline);
        assert_eq!(inline, shared);
        assert!(TextStr::from("a") < TextStr::from(String::from("b")));

        let set: HashSet<TextStr> = [literal].into_iter().collect();
        assert!(set.contains(&inline));
        assert!(set.contains(&shared));
        assert!(set.contains("Save"));
        assert_eq!(format!("{inline:?}"), "\"Save\"");
    }
}
