use crate::*;

pub trait Modifiers: View + Sized {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self>;
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self>;
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG>;
    fn geom<F: Fn(WorldRect) + 'static>(self, f: F) -> Geom<Self>;
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self>;
    fn offset(self, offset: LocalOffset) -> Offset<Self>;
}

impl<V: View + 'static> Modifiers for V {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self> {
        Tap::new(self, f)
    }
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG> {
        Background::new(self, background)
    }
    fn geom<F: Fn(WorldRect) + 'static>(self, f: F) -> Geom<Self> {
        Geom::new(self, f)
    }
    fn drag<F: Fn(LocalOffset, GestureState) + 'static>(self, f: F) -> Drag<Self> {
        Drag::new(self, f)
    }
    fn offset(self, offset: LocalOffset) -> Offset<Self> {
        Offset::new(self, offset)
    }
}
