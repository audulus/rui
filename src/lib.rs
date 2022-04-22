// #![feature(type_alias_impl_trait)]

use vger::color::*;
use vger::{LineMetrics, PaintIndex, Vger};

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

mod event_loop;
pub use event_loop::*;

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
