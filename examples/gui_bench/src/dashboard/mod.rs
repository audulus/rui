//! The app scenario: a sidebar with nav, a top bar with the search, the live switch and the
//! export menu, and one scroll view with the page of the active nav item: the overview (the
//! metric cards, the chart and filter cards and the orders table), the messages feed, or an empty
//! page (`apps/shared/dashboard.rs`).
//!
//! As rui teaches, the app state is one struct in a `state` at the root; every part gets its
//! handle and reads the state through the context. Each control that only needs a value of its
//! own, such as the hover of a row, keeps it in a `state` of its own. rui builds the whole view
//! tree again from these functions after every change.

mod chart;
mod filters;
mod kpis;
pub mod list;
mod live;
mod menu;
mod messages;
mod orders;
mod sidebar;
mod style;
mod topbar;

use std::sync::Arc;

use rui::*;

use crate::dash::{self, Page};
use style::{Surface, gap, label};

/// The state of the controls, shared by the parts that read or write it.
pub struct App {
    /// The active nav item.
    pub nav: usize,
    /// The text of the search input.
    pub search: String,
    /// The chart series, an index of `SEGMENTS`.
    pub series: usize,
    /// The compare switch.
    pub compare: bool,
    /// The status checkboxes, in the order of `Status::ALL`.
    pub shown: [bool; 4],
    /// Whether the orders are sorted by amount.
    pub sorted: bool,
    /// Whether the export menu is open.
    pub menu: bool,
    /// The live updates.
    pub live: live::Live,
}

impl App {
    fn new() -> Self {
        Self {
            nav: 0,
            search: String::new(),
            series: 0,
            compare: false,
            shown: [true; 4],
            sorted: false,
            menu: false,
            live: live::Live::default(),
        }
    }

    /// The Reset button: every status checked, compare off, the first series, no search, no
    /// sorting.
    pub fn reset(&mut self) {
        self.shown = [true; 4];
        self.compare = false;
        self.series = 0;
        self.search.clear();
        self.sorted = false;
    }
}

/// The data of the app, made once at startup and shared by the views that show it.
#[derive(Clone)]
pub struct Data {
    pub orders: Arc<Vec<orders::Row>>,
    /// The orders the table shows, found again only when the filter changes.
    pub shown: orders::Shown,
    pub messages: Arc<Vec<messages::Card>>,
    /// The indices of the messages, the keys of the feed.
    pub message_ids: Arc<Vec<usize>>,
}

/// Returns the window content with `count` orders and `count` messages.
pub fn view(count: usize) -> impl View {
    let data = Data {
        orders: Arc::new(
            dash::orders(count)
                .into_iter()
                .map(orders::Row::new)
                .collect(),
        ),
        shown: orders::Shown::default(),
        messages: Arc::new(
            dash::messages(count)
                .into_iter()
                .map(messages::Card::new)
                .collect(),
        ),
        message_ids: Arc::new((0..count).collect()),
    };
    state(App::new, move |app, cx| {
        let page = dash::page(cx[app].nav);
        let content = match page {
            Page::Overview => any_view(overview(app, data.clone())),
            Page::Messages => any_view(messages::view(
                data.messages.clone(),
                data.message_ids.clone(),
            )),
            Page::Empty => any_view(empty()),
        };
        let main = vstack((topbar::view(app), scroll(content).flex()));
        zstack((
            hstack((sidebar::view(app), main.flex()))
                .background(Surface::fill(dash::PAGE_BG, 0.0).view()),
            cond(
                cx[app].menu,
                any_view(menu::view(app)),
                any_view(EmptyView {}),
            ),
        ))
    })
}

/// The overview: the metric cards, the chart and filter cards, and the orders.
fn overview(app: StateHandle<App>, data: Data) -> impl View {
    vstack((
        kpis::view(app),
        gap(0.0, dash::GAP),
        hstack((
            chart::view(app).flex(),
            gap(dash::GAP, 0.0),
            filters::view(app),
        ))
        .fixed_height(dash::ROW2_HEIGHT),
        gap(0.0, dash::GAP),
        orders::view(app, data.orders, data.shown),
    ))
    .padding(dash::CONTENT_PADDING)
}

/// The empty page: its label, centered across the content.
fn empty() -> impl View {
    vstack((
        gap(0.0, dash::EMPTY_TOP),
        hstack((
            spacer(),
            label(dash::EMPTY_LABEL, dash::FONT_SIZE, dash::MUTED),
            spacer(),
        )),
    ))
}
