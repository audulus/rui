//! The list scenario: the header of the orders over a virtual list of order rows. rui has no
//! virtual list, so the app builds one: it keeps the scroll offset in its state, moves it with
//! `on_scroll`, and builds only the rows in the window and four more on each side, shifted by the
//! part of a row that is scrolled away and clipped to the space under the header.

use std::sync::Arc;

use rui::*;

use super::orders::{self, Row};
use super::style::Surface;
use crate::dash;

/// The rows built above and below the window.
const OVERSCAN: usize = 4;

/// The state of the list.
struct List {
    /// Whether the rows are sorted by amount.
    sorted: bool,
    /// The indices of the orders in the order of the rows.
    order: Arc<Vec<u32>>,
    /// How far the rows are scrolled, in pixels from the top.
    offset: f32,
    /// The height of the space under the header, from the last frame.
    viewport: f32,
}

/// Returns the window content: the header and the virtual list of `count` order rows.
pub fn view(count: usize) -> impl View {
    state(
        move || List {
            sorted: false,
            order: Arc::new(dash::list_order(count, false)),
            offset: 0.0,
            viewport: dash::WINDOW.1 - dash::LIST_ROWS_TOP,
        },
        move |list, cx| {
            let header = orders::header(cx[list].sorted, 0.0, move |cx| {
                let sorted = !cx[list].sorted;
                cx[list].sorted = sorted;
                cx[list].order = Arc::new(dash::list_order(count, sorted));
            });
            vstack((header, rows(list, cx).flex()))
                .background(Surface::fill(dash::CARD_BG, 0.0).view())
        },
    )
}

/// The rows in and near the space under the header.
fn rows(list: StateHandle<List>, cx: &Context) -> impl View + use<> {
    let state = &cx[list];
    let height = dash::ORDER_ROW_HEIGHT;
    let first = (state.offset / height) as usize;
    let visible = (state.viewport / height).ceil() as usize + 1;
    let start = first.saturating_sub(OVERSCAN);
    let end = (first + visible + OVERSCAN).min(state.order.len());
    let ids: Vec<u32> = state.order[start.min(end)..end].to_vec();
    let shift = state.offset - start as f32 * height;
    vstack((
        rui::list(ids, |&i| orders::row(&Row::new(dash::order(i as usize)))).offset([0.0, shift]),
        spacer(),
    ))
    .clip()
    .geom(move |cx, size, _| {
        if cx[list].viewport != size.height {
            cx[list].viewport = size.height;
        }
    })
    .on_scroll(move |cx, delta| {
        let total = cx[list].order.len() as f32 * dash::ORDER_ROW_HEIGHT;
        let max = (total - cx[list].viewport).max(0.0);
        let offset = (cx[list].offset + delta.y).clamp(0.0, max);
        if offset != cx[list].offset {
            cx[list].offset = offset;
        }
    })
}
