use crate::*;

pub trait Modifiers: View + Sized {
    /// Adds space around a view. Can be either `Auto` or `Px(number_of_pixels)`
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self>;

    /// Calls a function in response to a tap.
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self, F>;

    /// Puts a view behind another. The background view inherits the size of the view.
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG>;

    /// Calls a function with the view's geometry after layout runs.
    /// Currently only the view's size is returned.
    fn geom<F: Fn(LocalSize) + 'static>(self, f: F) -> Geom<Self, F>;

    /// Calls a function in response to a drag.
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self, F>;

    /// Applies an offset to the view in local space.
    fn offset<Off: Into<LocalOffset>>(self, offset: Off) -> Offset<Self>;

    /// Constrains the size of a view.
    fn size<Sz: Into<LocalSize>>(self, size: Sz) -> Size<Self>;
}

impl<V: View + 'static> Modifiers for V {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self, F> {
        Tap::new(self, f)
    }
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG> {
        Background::new(self, background)
    }
    fn geom<F: Fn(LocalSize) + 'static>(self, f: F) -> Geom<Self, F> {
        Geom::new(self, f)
    }
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self, F> {
        Drag::new(self, f)
    }
    fn offset<Off: Into<LocalOffset>>(self, offset: Off) -> Offset<Self> {
        Offset::new(self, offset.into())
    }
    fn size<Sz: Into<LocalSize>>(self, size: Sz) -> Size<Self> {
        Size::new(self, size.into())
    }
}

// Using this depends on https://github.com/rust-lang/rust/issues/63063
pub trait Body {
    type V: View;
    fn body(&self) -> Self::V;
}

impl<T> View for T
where
    T: Body,
{
    fn print(&self, id: ViewID, cx: &mut Context) {
        self.body().print(id, cx)
    }

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        self.body().needs_redraw(id, cx)
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.body().process(event, id, cx, vger)
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.body().draw(id, cx, vger)
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.body().layout(id, sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.body().hittest(id, pt, cx, vger)
    }
}

#[macro_export]
macro_rules! modifier_view {
    () => {
        fn print(&self, id: ViewID, cx: &mut Context) {
            self.body().print(id, cx)
        }

        fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
            self.body().needs_redraw(id, cx)
        }

        fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
            self.body().process(event, id, cx, vger)
        }

        fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
            self.body().draw(id, cx, vger)
        }

        fn layout(
            &self,
            id: ViewID,
            sz: LocalSize,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> LocalSize {
            self.body().layout(id, sz, cx, vger)
        }

        fn hittest(
            &self,
            id: ViewID,
            pt: LocalPoint,
            cx: &mut Context,
            vger: &mut VGER,
        ) -> Option<ViewID> {
            self.body().hittest(id, pt, cx, vger)
        }
    };
}
