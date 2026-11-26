//! The export menu. rui has no menu or popover, so the root of the window stacks this layer over
//! the app while the menu is open: a box of items under the export button. An item keeps its
//! hover in a `state` of its own; a click on it closes the menu.

use rui::*;

use super::App;
use super::style::{Surface, gap, label};
use crate::dash;

/// Returns the layer over the window that holds the open menu.
pub fn view(app: StateHandle<App>) -> impl View {
    let items = list((0..dash::MENU_ITEMS.len()).collect(), move |&i| {
        item(app, i)
    });
    let menu = vstack((
        gap(0.0, dash::MENU_PADDING),
        hstack((
            gap(dash::MENU_PADDING, 0.0),
            items.flex(),
            gap(dash::MENU_PADDING, 0.0),
        )),
        gap(0.0, dash::MENU_PADDING),
    ))
    .fixed_width(dash::MENU_WIDTH)
    .background(
        Surface::fill(dash::CARD_BG, 8.0)
            .border(dash::RULE)
            .shadow(dash::SHADOW_CARD)
            .view(),
    );
    vstack((
        gap(0.0, dash::MENU_Y),
        hstack((
            spacer(),
            menu,
            gap(dash::WINDOW.0 - dash::MENU_X - dash::MENU_WIDTH, 0.0),
        )),
        spacer(),
    ))
}

/// Menu item `i`.
fn item(app: StateHandle<App>, i: usize) -> impl View {
    state(
        || false,
        move |hover, cx| {
            let fill = cx[hover].then_some(dash::MENU_HOVER);
            hstack((
                gap(12.0, dash::MENU_ITEM_HEIGHT),
                label(dash::MENU_ITEMS[i], dash::FONT_SIZE, dash::TEXT),
                spacer(),
            ))
            .background(
                Surface {
                    fill,
                    radius: 6.0,
                    border: None,
                    shadow: None,
                }
                .view(),
            )
            // A tap does not pass events on, so the hover wraps it.
            .tap(move |cx| cx[app].menu = false)
            .hover(move |cx, inside| {
                if cx[hover] != inside {
                    cx[hover] = inside;
                }
            })
        },
    )
}
