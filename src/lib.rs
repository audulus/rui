// #![feature(type_alias_impl_trait)]

pub use vger;
use vger::color::*;
pub use vger::{LineMetrics, PaintIndex, Vger};

#[cfg(feature = "winit")]
#[macro_use]
extern crate lazy_static;

mod view;
pub use view::*;

mod viewid;
pub use viewid::*;

mod viewtuple;
pub use viewtuple::*;

mod event;
pub use event::*;

mod lens;
pub use lens::*;

mod binding;
pub use binding::*;

mod context;
pub use context::*;

mod views;
pub use views::*;

mod paint;
pub use paint::*;

mod modifiers;
pub use modifiers::*;

mod colors;
pub use colors::*;

mod align;
pub use align::*;

mod region;
pub use region::*;

#[cfg(feature = "winit")]
mod winit_event_loop;

#[cfg(feature = "winit")]
pub use winit_event_loop::*;

// See https://rust-lang.github.io/api-guidelines/future-proofing.html
pub(crate) mod private {
    pub trait Sealed {}
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_button() {
        let _ = button(text("click me"), |_cx| {
            println!("clicked!");
        });
    }
}
