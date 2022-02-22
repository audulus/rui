use crate::*;

pub trait Modifiers: View + Sized {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self>;
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self>;
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG>;
}

impl <V: View + 'static> Modifiers for V {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self> {
        Tap::new(self, f)
    }
    fn background<BG: View + 'static>(self, background: BG) -> Background<Self, BG> {
        Background::new(self, background)
    }
}