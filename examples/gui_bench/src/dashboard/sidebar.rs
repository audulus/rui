//! The sidebar: the brand, the nav items and the user block. A nav item keeps its hover in a
//! `state` of its own; a click makes it the active item.

use rui::*;

use super::App;
use super::style::{Surface, avatar, gap, label, leading, pill};
use crate::dash;

/// Returns the sidebar, `SIDEBAR_WIDTH` wide and as tall as the window.
pub fn view(app: StateHandle<App>) -> impl View {
    let items = list((0..dash::NAV_ITEMS.len()).collect(), move |&i| {
        // `NAV_STRIDE` minus `NAV_HEIGHT` of space under every item but the last.
        let below = if i + 1 < dash::NAV_ITEMS.len() {
            dash::NAV_STRIDE - dash::NAV_HEIGHT
        } else {
            0.0
        };
        vstack((
            hstack((
                gap(dash::NAV_X, 0.0),
                nav_item(app, i).flex(),
                gap(dash::NAV_X, 0.0),
            )),
            gap(0.0, below),
        ))
    });
    vstack((
        brand(),
        gap(0.0, dash::NAV_TOP - dash::TOPBAR_HEIGHT),
        items,
        spacer(),
        user(),
    ))
    .fixed_width(dash::SIDEBAR_WIDTH)
    .background(Surface::fill(dash::SIDEBAR_BG, 0.0).view())
}

/// The logo and the app name, as tall as the top bar.
fn brand() -> impl View {
    hstack((
        gap(20.0, dash::TOPBAR_HEIGHT),
        rectangle()
            .corner_radius(dash::LOGO_RADIUS)
            .color(super::style::color(dash::LOGO))
            .size([dash::LOGO_SIZE, dash::LOGO_SIZE]),
        gap(60.0 - 20.0 - dash::LOGO_SIZE, 0.0),
        label(dash::APP_NAME, dash::TITLE_FONT, 0xffffff),
        spacer(),
    ))
}

/// Nav item `i`: the icon, the label and, on the messages item, the badge.
fn nav_item(app: StateHandle<App>, i: usize) -> impl View {
    state(
        || false,
        move |hover, cx| {
            let active = cx[app].nav == i;
            let background = if active {
                Some(dash::NAV_ACTIVE)
            } else if cx[hover] {
                Some(dash::NAV_HOVER)
            } else {
                None
            };
            let icon = if active {
                dash::NAV_ICON_ACTIVE
            } else {
                dash::NAV_ICON_COLOR
            };
            let text = if active { 0xffffff } else { dash::SIDEBAR_TEXT };
            let badge = if i == dash::BADGE_ITEM {
                any_view(pill(
                    dash::BADGE_COUNT.to_string(),
                    12.0,
                    0xffffff,
                    20.0,
                    7.0,
                    dash::BADGE_BG,
                ))
            } else {
                any_view(EmptyView {})
            };
            hstack((
                gap(16.0, dash::NAV_HEIGHT),
                rectangle()
                    .corner_radius(4.0)
                    .color(super::style::color(icon))
                    .size([dash::NAV_ICON, dash::NAV_ICON]),
                gap(44.0 - 16.0 - dash::NAV_ICON, 0.0),
                label(dash::NAV_ITEMS[i], dash::FONT_SIZE, text),
                spacer(),
                badge,
                gap(12.0, 0.0),
            ))
            .background(
                Surface {
                    fill: background,
                    radius: dash::NAV_RADIUS,
                    border: None,
                    shadow: None,
                }
                .view(),
            )
            // A tap does not pass events on, so the hover wraps it.
            .tap(move |cx| {
                if cx[app].nav != i {
                    cx[app].nav = i;
                }
            })
            .hover(move |cx, inside| {
                if cx[hover] != inside {
                    cx[hover] = inside;
                }
            })
        },
    )
}

/// The user block at the bottom: a rule, then the avatar with the name and the role.
fn user() -> impl View {
    let names = vstack((
        leading(label(dash::USER_NAME, dash::FONT_SIZE, 0xffffff)),
        gap(0.0, 2.0),
        leading(label(dash::USER_ROLE, 12.0, dash::SIDEBAR_MUTED)),
    ));
    vstack((
        rectangle()
            .color(super::style::color(dash::SIDEBAR_RULE))
            .size([dash::SIDEBAR_WIDTH, 1.0]),
        gap(0.0, 740.0 - dash::USER_RULE_Y - 1.0),
        hstack((
            gap(20.0, 0.0),
            avatar(dash::USER_INITIALS, dash::USER_AVATAR, 12, dash::USER_COLOR),
            gap(64.0 - 20.0 - dash::USER_AVATAR, 0.0),
            names.flex(),
        )),
        gap(0.0, dash::WINDOW.1 - 740.0 - dash::USER_AVATAR),
    ))
}
