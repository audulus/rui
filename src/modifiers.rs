use crate::*;

pub trait Modifiers: View + Sized {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self>;
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self>;
}

impl <V: View + 'static> Modifiers for V {
    fn padding(self, param: impl Into<PaddingParam>) -> Padding<Self> {
        Padding::new(self, param.into())
    }
    fn tap<F: Fn() + 'static>(self, f: F) -> Tap<Self> {
        Tap::new(self, f)
    }
}