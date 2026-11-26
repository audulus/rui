//! The top bar: the page title, the search input, the live switch with its spinner, the export
//! button and the avatar. The controls right of the title keep their spec offsets from the right
//! edge, which puts them at their spec positions in a 1280 px window.

use rui::*;

use super::style::{self, Surface, avatar, button_box, color, gap, label, switch};
use super::{App, live};
use crate::dash;

/// Returns the top bar, `TOPBAR_HEIGHT` tall, its bottom rule included.
pub fn view(app: StateHandle<App>) -> impl View {
    state(
        || (),
        move |_, cx| {
            let title = dash::NAV_ITEMS[cx[app].nav];
            hstack((
                gap(
                    dash::TOPBAR_TITLE_X - dash::SIDEBAR_WIDTH,
                    dash::TOPBAR_HEIGHT,
                ),
                label(title, dash::TITLE_FONT + 4.0, dash::TEXT),
                spacer(),
                search(app),
                gap(dash::LIVE_X - dash::SEARCH_X - dash::SEARCH_WIDTH, 0.0),
                map(
                    cx[app].live.on,
                    move |on, cx| live::set(app, on, cx),
                    |on, _| switch(on),
                ),
                gap(dash::SPINNER_X - dash::LIVE_X - dash::SWITCH_WIDTH, 0.0),
                live::spinner(app),
                gap(dash::EXPORT_X - dash::SPINNER_X - dash::SPINNER_SIZE, 0.0),
                button_box(
                    dash::EXPORT_LABEL,
                    dash::EXPORT_WIDTH,
                    dash::PRIMARY_TEXT,
                    style::PRIMARY,
                )
                .tap(move |cx| cx[app].menu = !cx[app].menu),
                gap(
                    dash::TOPBAR_AVATAR_X - dash::EXPORT_X - dash::EXPORT_WIDTH,
                    0.0,
                ),
                avatar(
                    dash::USER_INITIALS,
                    dash::CONTROL_HEIGHT,
                    13,
                    dash::USER_COLOR,
                ),
                gap(1280.0 - dash::TOPBAR_AVATAR_X - dash::CONTROL_HEIGHT, 0.0),
            ))
            .background(canvas(|_, rect, vger| {
                let paint = vger.color_paint(color(dash::TOPBAR_BG));
                vger.fill_rect(rect, 0.0, paint);
                let rule = vger.color_paint(color(dash::RULE));
                vger.fill_rect(
                    LocalRect::new(rect.origin, [rect.width(), 1.0].into()),
                    0.0,
                    rule,
                );
            }))
        },
    )
}

/// The search input: rui's text editor in a bordered box, with the placeholder over it while
/// it is empty.
fn search(app: StateHandle<App>) -> impl View {
    let text = bind(app, SearchLens {});
    state(
        || (),
        move |_, cx| {
            let placeholder = if cx[app].search.is_empty() {
                any_view(label(
                    dash::SEARCH_PLACEHOLDER,
                    dash::FONT_SIZE,
                    dash::MUTED,
                ))
            } else {
                any_view(EmptyView {})
            };
            zstack((
                hstack((gap(12.0, 0.0), placeholder, spacer())).fixed_height(dash::CONTROL_HEIGHT),
                hstack((
                    gap(12.0, 0.0),
                    text_editor(text).size([256.0, 22.0]),
                    spacer(),
                ))
                .fixed_height(dash::CONTROL_HEIGHT),
            ))
            .size([dash::SEARCH_WIDTH, dash::CONTROL_HEIGHT])
            .background(
                Surface::fill(dash::INPUT_BG, 8.0)
                    .border(dash::INPUT_BORDER)
                    .view(),
            )
        },
    )
}

// The search text of the app state.
make_lens!(SearchLens, App, String, search);
