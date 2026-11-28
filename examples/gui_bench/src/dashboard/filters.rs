//! The filter card: the status checkboxes, the compare switch, and the Reset and Apply buttons.
//! The card has a fixed size, so its parts sit at their spec offsets in a `zstack`. rui has no
//! checkbox: a checkbox is a painted box in a row that toggles on a click.

use rui::*;

use super::App;
use super::style::{self, Surface, button_box, color, gap, label, leading, switch};
use crate::dash::{self, Status};

/// The top and the left of the card in the window.
const CARD_TOP: f32 = dash::TABLE_TOP - dash::GAP - dash::ROW2_HEIGHT;
const CARD_X: f32 = 1280.0 - dash::CONTENT_PADDING - dash::FILTERS_WIDTH;

/// Places `view` at the window point (`x`, `y`) with size (`w`, `h`) in the card. rui's y is up,
/// so the offset counts from the card's bottom.
fn place(view: impl View, x: f32, y: f32, w: f32, h: f32) -> impl View {
    view.size([w, h])
        .offset([x - CARD_X, CARD_TOP + dash::ROW2_HEIGHT - y - h])
}

/// Returns the filter card.
pub fn view(app: StateHandle<App>) -> impl View {
    memo(move |cx| {
        let shown = cx[app].shown;
        let compare = cx[app].compare;
        let checks = zlist(Status::ALL.to_vec(), move |&status| {
            let k = status.filter_row();
            place(
                check_row(app, k, shown[k], status.label()),
                dash::CHECK_X,
                dash::CHECK_TOP + k as f32 * dash::CHECK_STRIDE,
                dash::SWITCH_X + dash::SWITCH_WIDTH - dash::CHECK_X,
                dash::CHECK_STRIDE,
            )
        });
        zstack((
            place(
                leading(label(dash::FILTERS_TITLE, dash::TITLE_FONT, dash::TEXT)),
                CARD_X + dash::CARD_PADDING,
                CARD_TOP + dash::CARD_PADDING,
                200.0,
                20.0,
            ),
            checks,
            place(
                leading(label(dash::COMPARE_LABEL, dash::FONT_SIZE, dash::TEXT)),
                dash::CHECK_X,
                dash::COMPARE_Y,
                240.0,
                28.0,
            ),
            place(
                map(
                    compare,
                    move |on, cx| cx[app].compare = on,
                    |on, _| switch(on),
                ),
                dash::SWITCH_X,
                dash::SWITCH_Y,
                dash::SWITCH_WIDTH,
                dash::SWITCH_HEIGHT,
            ),
            place(
                button_box(
                    dash::RESET_LABEL,
                    dash::BUTTON_WIDTH,
                    dash::TEXT,
                    style::SECONDARY,
                )
                .tap(move |cx| cx[app].reset()),
                dash::RESET_X,
                dash::BUTTONS_Y,
                dash::BUTTON_WIDTH,
                dash::CONTROL_HEIGHT,
            ),
            place(
                button_box(
                    dash::APPLY_LABEL,
                    dash::BUTTON_WIDTH,
                    dash::PRIMARY_TEXT,
                    style::PRIMARY,
                )
                .tap(|_| ()),
                dash::APPLY_X,
                dash::BUTTONS_Y,
                dash::BUTTON_WIDTH,
                dash::CONTROL_HEIGHT,
            ),
        ))
        .size([dash::FILTERS_WIDTH, dash::ROW2_HEIGHT])
        .background(Surface::CARD.view())
    })
}

/// The row of checkbox `k`: the box and the label. A click anywhere on the row toggles it.
fn check_row(
    app: StateHandle<App>,
    k: usize,
    checked: bool,
    text: &'static str,
) -> impl View + use<> {
    hstack((
        checkbox(checked),
        gap(28.0 - dash::CHECK_SIZE, dash::CHECK_STRIDE),
        label(text, dash::FONT_SIZE, dash::TEXT),
        spacer(),
    ))
    .background(
        Surface {
            fill: None,
            radius: 0.0,
            border: None,
            shadow: None,
        }
        .view(),
    )
    .tap(move |cx| cx[app].shown[k] = !cx[app].shown[k])
}

/// The box of a checkbox: a filled box with a check mark, or an empty one with a border.
fn checkbox(checked: bool) -> impl View {
    canvas(move |_, rect, vger| {
        if checked {
            Surface::fill(dash::PRIMARY, 4.0).paint(rect, vger);
            let mark = vger.color_paint(color(0xffffff));
            let s = dash::CHECK_SIZE;
            let (a, b, c) = (
                LocalPoint::new(s * 0.25, s * 0.52),
                LocalPoint::new(s * 0.43, s * 0.32),
                LocalPoint::new(s * 0.75, s * 0.70),
            );
            vger.stroke_segment(a, b, 1.0, mark);
            vger.stroke_segment(b, c, 1.0, mark);
        } else {
            Surface::fill(0xffffff, 4.0)
                .border(dash::INPUT_BORDER)
                .paint(rect, vger);
        }
    })
    .size([dash::CHECK_SIZE, dash::CHECK_SIZE])
}
