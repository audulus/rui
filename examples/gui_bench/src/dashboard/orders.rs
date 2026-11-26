//! The orders card: the column header and a `list` of the orders that pass the status filter and
//! the search, in the order of the data or by amount. Every shown order is a real row view,
//! keyed by its index in the data, and keeps its hover in a `state` of its own.

use std::sync::Arc;

use rui::*;

use super::App;
use super::style::{Surface, avatar, color, gap, label, leading, pill, trailing};
use crate::dash::{self, Order};

/// The widths of the order, customer, status and progress columns, and of the amount column,
/// which ends `COL_AMOUNT_END` from the left of a 992 px card.
const ORDER_WIDTH: f32 = dash::COL_CUSTOMER - dash::COL_ORDER;
const CUSTOMER_WIDTH: f32 = dash::COL_STATUS - dash::COL_CUSTOMER;
const STATUS_WIDTH: f32 = dash::COL_PROGRESS - dash::COL_STATUS;
const AMOUNT_WIDTH: f32 = dash::SORT_WIDTH;
/// The space right of the amount column.
const PADDING_RIGHT: f32 = 992.0 - dash::COL_AMOUNT_END;

/// One order with the texts its row shows, made once and shared by every build of the row.
pub struct Row {
    pub order: Order,
    number: Arc<str>,
    customer: Arc<str>,
    initials: Arc<str>,
    amount: Arc<str>,
}

impl Row {
    pub fn new(order: Order) -> Self {
        Self {
            number: order.number_text().into(),
            customer: order.customer.as_str().into(),
            initials: order.initials.as_str().into(),
            amount: order.amount().into(),
            order,
        }
    }
}

/// Returns the orders card.
pub fn view(app: StateHandle<App>, orders: Arc<Vec<Row>>) -> impl View {
    state(
        || (),
        move |_, cx| {
            let state = &cx[app];
            let mut ids: Vec<usize> = (0..orders.len())
                .filter(|&i| dash::visible(&orders[i].order, &state.shown, &state.search))
                .collect();
            if state.sorted {
                ids.sort_by(|a, b| orders[*b].order.cents.cmp(&orders[*a].order.cents));
            }
            let orders = orders.clone();
            vstack((
                header(state.sorted, dash::CARD_RADIUS, move |cx| {
                    cx[app].sorted = !cx[app].sorted
                }),
                list(ids, move |&i| row(&orders[i])),
            ))
            .background(Surface::CARD.view())
        },
    )
}

/// The header with the column titles, its top corners rounded by `radius`. A click on the amount
/// title calls `sort`; the title is in the primary color while the orders are `sorted`.
pub fn header<F: Fn(&mut Context) + Clone + 'static>(
    sorted: bool,
    radius: f32,
    sort: F,
) -> impl View + use<F> {
    let title = |k: usize| label(dash::COLUMNS[k], 12.0, dash::MUTED);
    let amount = if sorted { dash::PRIMARY } else { dash::MUTED };
    hstack((
        gap(dash::COL_ORDER, dash::TABLE_HEADER_HEIGHT),
        leading(title(0)).fixed_width(ORDER_WIDTH),
        leading(title(1)).fixed_width(CUSTOMER_WIDTH),
        leading(title(2)).fixed_width(STATUS_WIDTH),
        leading(title(3)).flex(),
        trailing(label(dash::COLUMNS[dash::SORT_COLUMN], 12.0, amount))
            .fixed_width(AMOUNT_WIDTH)
            .fixed_height(dash::TABLE_HEADER_HEIGHT)
            .background(
                Surface {
                    fill: None,
                    radius: 0.0,
                    border: None,
                    shadow: None,
                }
                .view(),
            )
            .tap(move |cx| sort(cx)),
        gap(PADDING_RIGHT, 0.0),
    ))
    .background(canvas(move |_, rect, vger| {
        let fill = vger.color_paint(color(dash::TABLE_HEADER_BG));
        vger.fill_rect(rect, radius, fill);
        // Square bottom corners.
        let half = LocalRect::new(rect.origin, [rect.width(), rect.height() / 2.0].into());
        vger.fill_rect(half, 0.0, fill);
        let rule = vger.color_paint(color(dash::RULE));
        vger.fill_rect(
            LocalRect::new(rect.origin, [rect.width(), 1.0].into()),
            0.0,
            rule,
        );
    }))
}

/// The row of an order. The pointer over it gives it the hover color at once.
pub fn row(row: &Row) -> impl View + use<> {
    let order = &row.order;
    let (pill_bg, pill_fg) = order.status.colors();
    let cells = hstack((
        gap(dash::COL_ORDER, dash::ORDER_ROW_HEIGHT),
        leading(label(&row.number, dash::FONT_SIZE, dash::MUTED)).fixed_width(ORDER_WIDTH),
        leading(hstack((
            avatar(row.initials.clone(), dash::AVATAR, 11, order.avatar),
            gap(40.0 - dash::AVATAR, 0.0),
            label(&row.customer, dash::FONT_SIZE, dash::TEXT),
        )))
        .fixed_width(CUSTOMER_WIDTH),
        leading(pill(
            order.status.label(),
            12.0,
            pill_fg,
            22.0,
            10.0,
            pill_bg,
        ))
        .fixed_width(STATUS_WIDTH),
        progress(order.progress),
        spacer(),
        trailing(label(&row.amount, dash::FONT_SIZE, dash::TEXT)).fixed_width(AMOUNT_WIDTH),
        gap(PADDING_RIGHT, 0.0),
    ));
    state(
        || false,
        move |hover, cx| {
            let fill = if cx[hover] {
                dash::ROW_HOVER
            } else {
                dash::CARD_BG
            };
            cells
                .clone()
                .background(canvas(move |_, rect, vger| {
                    let paint = vger.color_paint(color(fill));
                    vger.fill_rect(rect, 0.0, paint);
                    let rule = vger.color_paint(color(dash::ROW_RULE));
                    vger.fill_rect(
                        LocalRect::new(rect.origin, [rect.width(), 1.0].into()),
                        0.0,
                        rule,
                    );
                }))
                .hover(move |cx, inside| {
                    if cx[hover] != inside {
                        cx[hover] = inside;
                    }
                })
        },
    )
}

/// The progress bar: a track with a fill of `percent`.
fn progress(percent: u32) -> impl View {
    canvas(move |_, rect, vger| {
        let radius = dash::PROGRESS_HEIGHT / 2.0;
        let track = vger.color_paint(color(dash::TRACK));
        vger.fill_rect(rect, radius, track);
        if percent > 0 {
            let fill = vger.color_paint(color(dash::PRIMARY));
            let width = rect.width() * percent as f32 / 100.0;
            vger.fill_rect(
                LocalRect::new(
                    rect.origin,
                    [width.max(dash::PROGRESS_HEIGHT), rect.height()].into(),
                ),
                radius,
                fill,
            );
        }
    })
    .size([dash::PROGRESS_WIDTH, dash::PROGRESS_HEIGHT])
}
