// Vendored from gui-bench (https://github.com/zortax/gui-bench), apps/shared/dashboard.rs,
// as of audulus/gui-bench@329923a. Copyright the gui-bench authors; licensed under the Apache
// License 2.0 (see examples/gui_bench/LICENSE-APACHE). Keep it in sync with gui-bench, so the
// app draws what the benchmark's probes expect.

//! The `app` scenario: a realistic analytics app with a sidebar, a top bar, metric cards, a
//! bar chart with controls, and a table of orders, styled with rounded corners and shadows. Every
//! app draws the same layout, so the probes of the runner (`runner/src/dashboard.rs`) find the same
//! elements at the same window points. Measures are logical pixels at scale 1, colors `0xrrggbb`.
//!
//! The window is 1280x800:
//!
//! - **Sidebar**, `x 0..240`, full height, dark: the brand at the top, nine nav items, and the
//!   user at the bottom. The pointer over a nav item that is not active gives it `NAV_HOVER`.
//!   A click makes the item active.
//! - **Top bar**, `x 240..1280, y 0..64`, its 1 px bottom border included: the page title, a search
//!   input, the export button and the avatar.
//! - **Content**, `x 240..1280, y 64..800`: one vertical scroll container with `CONTENT_PADDING`
//!   on every side and `GAP` between its blocks. It scrolls the way the framework's scroll
//!   container does by default, like the `rows` scenario. It holds, from the top:
//!   1. four metric cards in a row, `KPI_HEIGHT` tall, which share the width;
//!   2. the chart card (`CHART_WIDTH`, flexible) and the filter card (`FILTERS_WIDTH`, fixed),
//!      `ROW2_HEIGHT` tall;
//!   3. the orders card: a header and `--rows` order rows (all of them real elements, no
//!      virtualization), each with an avatar, a status pill and a progress bar.
//!
//! The nav items switch the page that the content shows (`Page`): "Overview" is the page above,
//! "Messages" is a feed of `--rows` message cards with wrapping text, and every other item shows
//! an empty page. The top bar also holds the live switch with its spinner, and the export button
//! opens a menu over the content. A click on the "AMOUNT" column title sorts the orders.
//!
//! The `list` scenario shows the orders alone, in the framework's virtual list; see
//! `LIST_ROWS_TOP`.
//!
//! The functions below give every text and number. A window narrower than 1280 shrinks the
//! flexible widths (the metric cards, the chart card, the orders card); content that no longer
//! fits may clip.

#![allow(dead_code)]

/// The window size in logical pixels.
pub const WINDOW: (f32, f32) = (1280.0, 800.0);

// ---- Geometry -------------------------------------------------------------------------------

/// The width of the sidebar.
pub const SIDEBAR_WIDTH: f32 = 240.0;
/// The height of the top bar, its 1 px bottom border included.
pub const TOPBAR_HEIGHT: f32 = 64.0;
/// The padding of the content on every side.
pub const CONTENT_PADDING: f32 = 24.0;
/// The space between two cards, across and down.
pub const GAP: f32 = 16.0;
/// The corner radius of every card.
pub const CARD_RADIUS: f32 = 12.0;
/// The padding inside the metric, chart and filter cards.
pub const CARD_PADDING: f32 = 20.0;

/// The brand: a `LOGO_SIZE` square with `LOGO_RADIUS` at (`20`, `18`), and the app name in
/// `TITLE_FONT` at x `60`, centered on y `32`.
pub const LOGO_SIZE: f32 = 28.0;
pub const LOGO_RADIUS: f32 = 8.0;
/// The nav items: item `i` covers `x 12..228`, `y NAV_TOP + i * NAV_STRIDE` for `NAV_HEIGHT`, with
/// `NAV_RADIUS`. Its icon is a `NAV_ICON` square with radius 4 at item x + 16, centered; its label
/// starts at item x + 44 (window x 56).
pub const NAV_X: f32 = 12.0;
pub const NAV_WIDTH: f32 = 216.0;
pub const NAV_TOP: f32 = 80.0;
pub const NAV_HEIGHT: f32 = 40.0;
pub const NAV_STRIDE: f32 = 44.0;
pub const NAV_RADIUS: f32 = 8.0;
pub const NAV_ICON: f32 = 16.0;
/// The nav item with the unread badge, and its count. The badge is a pill of height 20, radius 10,
/// padding 0 7, `BADGE_BG`, 12 px semibold white text, right-aligned 12 px inside the item.
pub const BADGE_ITEM: usize = 6;
pub const BADGE_COUNT: u32 = 3;
/// The user block at the bottom of the sidebar: a 1 px `SIDEBAR_RULE` line over `x 0..240` at
/// y `720`, then a `USER_AVATAR` circle at (`20`, `740`) with the initials, the name in 14 px white
/// and the role in 12 px `SIDEBAR_MUTED` beside it at x `64`.
pub const USER_RULE_Y: f32 = 720.0;
pub const USER_AVATAR: f32 = 32.0;

/// The top bar title in `TITLE_FONT` + 4 at x `264`, centered on y `32`.
pub const TOPBAR_TITLE_X: f32 = 264.0;
/// The search input: the framework's single-line text input, `x 676..956`, `y 14..50`, radius 8,
/// a 1 px `INPUT_BORDER` border, `INPUT_BG` fill and the placeholder `SEARCH_PLACEHOLDER`. Typing
/// filters the orders by customer name, case-insensitively. It is not focused at startup.
pub const SEARCH_X: f32 = 676.0;
pub const SEARCH_WIDTH: f32 = 280.0;
pub const CONTROL_HEIGHT: f32 = 36.0;
/// The export button, `x 1100..1196`, `y 14..50`: a primary button.
pub const EXPORT_X: f32 = 1100.0;
pub const EXPORT_WIDTH: f32 = 96.0;
/// The avatar of the top bar: a 36 px `USER_COLOR` circle at x `1220`, centered, with the initials
/// in 13 px semibold white.
pub const TOPBAR_AVATAR_X: f32 = 1220.0;

/// The metric cards: card `k` covers `x 264 + k * 252` for `KPI_WIDTH`, `y 88..200`.
pub const KPI_COUNT: usize = 4;
pub const KPI_HEIGHT: f32 = 112.0;
/// The width of a metric card in a 1280 px window: (992 - 3 * 16) / 4.
pub const KPI_WIDTH: f32 = 236.0;
/// Inside a metric card: the label (13 px `MUTED`) at (20, 18), the value (24 px bold `TEXT`) at
/// (20, 40), the delta pill at (20, 78), 22 tall, radius 11, padding 0 8, 12 px semibold. The
/// sparkline is `SPARK_BARS` bars of `SPARK_BAR_WIDTH` every `SPARK_STEP`, radius 2, `SPARK`, their
/// bottoms on card y 92 and their right end 20 px inside the card.
pub const SPARK_BARS: usize = 12;
pub const SPARK_BAR_WIDTH: f32 = 6.0;
pub const SPARK_STEP: f32 = 9.0;

/// The second row: `y 216..536`.
pub const ROW2_HEIGHT: f32 = 320.0;
/// The chart card, `x 264..904` in a 1280 px window.
pub const CHART_WIDTH: f32 = 640.0;
/// The filter card, `x 920..1256`.
pub const FILTERS_WIDTH: f32 = 336.0;

/// Inside the chart card: the title `"Revenue"` (16 px semibold) at (20, 20). The segmented
/// control at the top right: a `SEGMENT_TRACK` box with radius 8 and 2 px padding, `x 688..884`,
/// `y 232..264` in the window, holding three segments of 64x28 (`SEGMENTS`); the selected one is
/// white with radius 6 and `SHADOW_SMALL`, its text `TEXT`, the others `MUTED`, 13 px semibold. A
/// click on a segment selects it and shows its series.
pub const SEGMENT_WIDTH: f32 = 64.0;
pub const SEGMENT_HEIGHT: f32 = 28.0;
/// The plot: `x 284..884`, `y 288..516` in the window, with the baseline at y 516. Four 1 px
/// `GRID` lines at y 300, 354, 408 and 462 behind the bars. `CHART_BARS` slots of `CHART_SLOT`;
/// bar `i` of the shown series takes slot x + 4 for 17 px, its height `bar_height(series, i)`,
/// its top corners radius 4 (or all four), color `BAR`. With "compare" on, the slot holds the
/// previous period at slot x + 3 for 8 px (`BAR_PREVIOUS`, height `previous_height`) and the
/// current one at slot x + 13 for 8 px.
pub const PLOT_X: f32 = 284.0;
pub const PLOT_TOP: f32 = 288.0;
pub const PLOT_BOTTOM: f32 = 516.0;
pub const CHART_BARS: usize = 24;
pub const CHART_SLOT: f32 = 25.0;
pub const GRID_LINES: [f32; 4] = [300.0, 354.0, 408.0, 462.0];

/// Inside the filter card (`x 920..1256`, `y 216..536`): the title `"Filters"` (16 px semibold)
/// at (20, 20); the four status checkboxes (the framework's checkbox, checked at startup), row `k`
/// at window `y 272 + 32 k` for 32 px, the box (18 px) at window x 940, centered on the row, the
/// label (14 px `TEXT`) at x 968. Unchecking a status hides the orders with it.
pub const CHECK_TOP: f32 = 272.0;
pub const CHECK_STRIDE: f32 = 32.0;
pub const CHECK_X: f32 = 940.0;
pub const CHECK_SIZE: f32 = 18.0;
/// The compare row, `y 412..440`: the label `COMPARE_LABEL` (14 px `TEXT`) at x 940 and the
/// switch, `x 1196..1236`, `y 415..437` (40x22, radius 11). Off: `SWITCH_OFF` track and a white
/// 18 px knob 2 px from the left. On: `PRIMARY` track and the knob on the right. It starts off. A
/// click on the switch toggles it.
pub const COMPARE_Y: f32 = 412.0;
pub const SWITCH_X: f32 = 1196.0;
pub const SWITCH_Y: f32 = 415.0;
pub const SWITCH_WIDTH: f32 = 40.0;
pub const SWITCH_HEIGHT: f32 = 22.0;
/// The buttons, `y 480..516`: "Reset" (secondary) at `x 1024..1124`, "Apply" (primary) at
/// `x 1136..1236`. Reset checks every status, turns compare off, selects the first segment and
/// clears the search; Apply does nothing.
pub const BUTTONS_Y: f32 = 480.0;
pub const BUTTON_WIDTH: f32 = 100.0;
pub const RESET_X: f32 = 1024.0;
pub const APPLY_X: f32 = 1136.0;

/// The orders card: `x 264..1256` from y 552. A header of `TABLE_HEADER_HEIGHT` (its 1 px bottom
/// border `RULE` included, `TABLE_HEADER_BG`, top corners radius 12) with the column titles in
/// 12 px semibold `MUTED` capitals, then the rows: row `r` at window `y 596 + 52 r` before any
/// scroll, `ORDER_ROW_HEIGHT` tall, its 1 px bottom border `ROW_RULE` included. The pointer over a
/// row gives it `ROW_HOVER`, at once, with no transition.
pub const TABLE_TOP: f32 = 552.0;
pub const TABLE_HEADER_HEIGHT: f32 = 44.0;
pub const ORDER_ROW_HEIGHT: f32 = 52.0;
/// The columns, from the left of the card: the order number (14 px `MUTED`), the customer (a
/// 28 px avatar circle with 11 px semibold white initials, then the name at +40 in 14 px `TEXT`),
/// the status pill (22 tall, radius 11, padding 0 10, 12 px semibold), the progress bar (120x6,
/// radius 3, `TRACK` with a `PRIMARY` fill), and the amount (14 px semibold `TEXT`), which ends
/// `COL_AMOUNT_END` from the left.
pub const COL_ORDER: f32 = 24.0;
pub const COL_CUSTOMER: f32 = 140.0;
pub const COL_STATUS: f32 = 440.0;
pub const COL_PROGRESS: f32 = 600.0;
pub const COL_AMOUNT_END: f32 = 968.0;
pub const PROGRESS_WIDTH: f32 = 120.0;
pub const PROGRESS_HEIGHT: f32 = 6.0;
pub const AVATAR: f32 = 28.0;
pub const COLUMNS: [&str; 5] = ["ORDER", "CUSTOMER", "STATUS", "PROGRESS", "AMOUNT"];

/// A click on the "AMOUNT" column title sorts the orders by amount, largest first; the next click
/// brings the order of the data back. While the orders are sorted the title is `PRIMARY`. The
/// title is the only one that reacts, over `x SORT_X..SORT_X + SORT_WIDTH` of the card.
pub const SORT_COLUMN: usize = 4;
pub const SORT_X: f32 = 848.0;
pub const SORT_WIDTH: f32 = 120.0;

/// The live switch of the top bar, `x 976..1016`, `y 21..43`: the framework's switch, the same
/// look as the compare switch, off at startup. While it is on, a timer ticks every
/// `LIVE_INTERVAL_MS`, every metric card shows `kpi_value(k, tick)`, and the spinner shows: a
/// `SPINNER_SIZE` square with radius `SPINNER_RADIUS` in `PRIMARY` at x `1040`, centered on y 32,
/// which turns once every `SPINNER_PERIOD_MS` at a constant speed. Turning the switch off hides
/// the spinner, stops the timer and shows the startup values again.
pub const LIVE_X: f32 = 976.0;
pub const LIVE_Y: f32 = 21.0;
pub const LIVE_INTERVAL_MS: u64 = 100;
pub const SPINNER_X: f32 = 1040.0;
pub const SPINNER_SIZE: f32 = 24.0;
pub const SPINNER_RADIUS: f32 = 6.0;
pub const SPINNER_PERIOD_MS: u64 = 1000;

/// The export menu: a click on the export button opens it, and a click on the button or on an
/// item closes it. It lies over the content, its left edge under the button's, `x 1100..1260` from y `MENU_Y`, with `MENU_PADDING`
/// around `MENU_ITEMS` items of `MENU_ITEM_HEIGHT`: a `CARD_BG` box with radius 8, a 1 px `RULE`
/// border and `SHADOW_CARD`. An item holds its label (14 px `TEXT`) 12 px from its left, takes
/// `MENU_HOVER` and radius 6 under the pointer, and does nothing else. The menu is the
/// framework's menu or popover where it has one.
pub const MENU_X: f32 = 1100.0;
pub const MENU_Y: f32 = 54.0;
pub const MENU_WIDTH: f32 = 160.0;
pub const MENU_PADDING: f32 = 4.0;
pub const MENU_ITEM_HEIGHT: f32 = 36.0;

/// The messages page: the scroll container of the content holds `--rows` message cards, one per
/// row, `MESSAGE_GAP` apart. A card has the look of every card, padding `MESSAGE_PADDING` (down,
/// across), and holds a `MESSAGE_AVATAR` circle with 13 px semibold white initials at its top
/// left; `MESSAGE_TEXT_X` from the card's left the author (14 px semibold `TEXT`) and, 8 px
/// after it, the time (13 px `MUTED`) on one line of `MESSAGE_LINE`; and below that the body
/// (14 px `TEXT`, lines of `MESSAGE_LINE`), which wraps at the right padding of the card. The
/// height of a card follows its body, so a narrower window makes the cards taller.
pub const MESSAGE_GAP: f32 = 12.0;
pub const MESSAGE_PADDING: (f32, f32) = (16.0, 20.0);
pub const MESSAGE_AVATAR: f32 = 36.0;
pub const MESSAGE_TEXT_X: f32 = 68.0;
pub const MESSAGE_LINE: f32 = 20.0;

/// The empty page of the nav items without content: `EMPTY_LABEL` in 14 px `MUTED`, centered
/// across the content, its top `CONTENT_PADDING` + 48 below the top bar.
pub const EMPTY_TOP: f32 = 72.0;

/// The `list` scenario: the window holds the header of the orders card over its full width at
/// the top (`TABLE_HEADER_HEIGHT`, no radius, no shadow) and below it the framework's virtual
/// list, which fills the rest of the window with `--rows` order rows of `ORDER_ROW_HEIGHT`: row
/// `r` at window `y LIST_ROWS_TOP + 52 r` before any scroll. Only the rows in and near the window
/// exist. The rows and the column titles look as in the app and keep their offsets from the left
/// edge; the amount column of the header ends 24 px from the right edge, and its title sorts
/// over the last `SORT_WIDTH` before that. The amounts of the rows end where the scrollbar of the
/// list leaves them, at most 24 px further left. Row `r` shows `order(list_order(count, sorted)[r])`. The list scrolls
/// the way the framework's virtual list does by default.
pub const LIST_ROWS_TOP: f32 = 44.0;

/// The font sizes: 14 px unless the layout says otherwise. All text is sans-serif; "semibold" is
/// weight 600 and "bold" weight 700 where the framework has them.
pub const FONT_SIZE: f32 = 14.0;
pub const TITLE_FONT: f32 = 16.0;

/// The shadow of every card: offset (0, 4), blur 12, `SHADOW` at 10 % alpha.
pub const SHADOW_CARD: Shadow = Shadow {
    x: 0.0,
    y: 4.0,
    blur: 12.0,
    color: SHADOW,
    alpha: 0.10,
};
/// The shadow of the primary buttons and the switch knob: offset (0, 1), blur 3, at 25 % alpha.
pub const SHADOW_SMALL: Shadow = Shadow {
    x: 0.0,
    y: 1.0,
    blur: 3.0,
    color: SHADOW,
    alpha: 0.25,
};

/// A drop shadow.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Shadow {
    pub x: f32,
    pub y: f32,
    pub blur: f32,
    pub color: u32,
    pub alpha: f32,
}

// ---- Colors ---------------------------------------------------------------------------------

pub const PAGE_BG: u32 = 0xf3f4f6;
pub const CARD_BG: u32 = 0xffffff;
pub const TEXT: u32 = 0x111827;
pub const MUTED: u32 = 0x6b7280;
pub const RULE: u32 = 0xe5e7eb;
pub const SHADOW: u32 = 0x111827;

pub const SIDEBAR_BG: u32 = 0x111827;
pub const SIDEBAR_TEXT: u32 = 0xd1d5db;
pub const SIDEBAR_MUTED: u32 = 0x9ca3af;
pub const SIDEBAR_RULE: u32 = 0x1f2937;
pub const NAV_ACTIVE: u32 = 0x1f2937;
pub const NAV_HOVER: u32 = 0x374151;
pub const NAV_ICON_COLOR: u32 = 0x4b5563;
pub const NAV_ICON_ACTIVE: u32 = 0x818cf8;
pub const LOGO: u32 = 0x6366f1;
pub const BADGE_BG: u32 = 0xef4444;

pub const TOPBAR_BG: u32 = 0xffffff;
pub const INPUT_BG: u32 = 0xf9fafb;
pub const INPUT_BORDER: u32 = 0xd1d5db;
pub const PRIMARY: u32 = 0x4f46e5;
pub const PRIMARY_TEXT: u32 = 0xffffff;
pub const SECONDARY_BG: u32 = 0xffffff;
pub const SECONDARY_BORDER: u32 = 0xd1d5db;
pub const USER_COLOR: u32 = 0xf59e0b;

pub const SPARK: u32 = 0xc7d2fe;
pub const UP_BG: u32 = 0xdcfce7;
pub const UP_TEXT: u32 = 0x166534;
pub const DOWN_BG: u32 = 0xfee2e2;
pub const DOWN_TEXT: u32 = 0x991b1b;

pub const SEGMENT_TRACK: u32 = 0xf3f4f6;
pub const GRID: u32 = 0xeef0f3;
pub const BAR: u32 = 0x6366f1;
pub const BAR_PREVIOUS: u32 = 0xc7d2fe;
pub const SWITCH_OFF: u32 = 0xd1d5db;

pub const TABLE_HEADER_BG: u32 = 0xf9fafb;
pub const ROW_RULE: u32 = 0xf3f4f6;
pub const ROW_HOVER: u32 = 0xeef2ff;
pub const TRACK: u32 = 0xe5e7eb;
pub const MENU_HOVER: u32 = 0xf3f4f6;

/// The avatar colors of the customers.
pub const AVATAR_COLORS: [u32; 8] = [
    0xf87171, 0xfb923c, 0xfbbf24, 0x34d399, 0x22d3ee, 0x60a5fa, 0xa78bfa, 0xf472b6,
];

// ---- Texts and data -------------------------------------------------------------------------

pub const APP_NAME: &str = "Acme Analytics";
pub const PAGE_TITLE: &str = "Overview";
pub const USER_NAME: &str = "Ada Lovelace";
pub const USER_ROLE: &str = "Admin";
pub const USER_INITIALS: &str = "AL";
pub const SEARCH_PLACEHOLDER: &str = "Search customers";
pub const EXPORT_LABEL: &str = "Export";
pub const CHART_TITLE: &str = "Revenue";
pub const FILTERS_TITLE: &str = "Filters";
pub const COMPARE_LABEL: &str = "Compare to previous period";
pub const RESET_LABEL: &str = "Reset";
pub const APPLY_LABEL: &str = "Apply";
pub const LIVE_LABEL: &str = "Live updates";
pub const EMPTY_LABEL: &str = "Nothing here yet";

/// The items of the export menu, top to bottom.
pub const MENU_ITEMS: [&str; 3] = ["Export CSV", "Export PDF", "Print"];

/// What the content shows; the active nav item picks it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Page {
    /// The metric cards, the chart, the filters and the orders.
    Overview,
    /// The feed of message cards.
    Messages,
    /// Nothing but `EMPTY_LABEL`.
    Empty,
}

/// The nav item of the messages page, which is also the item with the badge.
pub const MESSAGES_ITEM: usize = BADGE_ITEM;

/// Returns the page of nav item `item`.
pub fn page(item: usize) -> Page {
    match item {
        0 => Page::Overview,
        MESSAGES_ITEM => Page::Messages,
        _ => Page::Empty,
    }
}

/// The nav items, the first active at startup.
pub const NAV_ITEMS: [&str; 9] = [
    "Overview",
    "Orders",
    "Customers",
    "Products",
    "Analytics",
    "Reports",
    "Messages",
    "Settings",
    "Help",
];

/// The segments of the chart, the first selected at startup.
pub const SEGMENTS: [&str; 3] = ["Day", "Week", "Month"];

/// One metric card.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Kpi {
    pub label: &'static str,
    pub value: &'static str,
    pub delta: &'static str,
    /// True for a rise: `UP_BG`/`UP_TEXT`; false: `DOWN_BG`/`DOWN_TEXT`.
    pub up: bool,
    /// The sparkline bar heights in px, oldest first.
    pub spark: [f32; SPARK_BARS],
}

/// The metric cards, left to right.
pub const KPIS: [Kpi; KPI_COUNT] = [
    Kpi {
        label: "Total revenue",
        value: "$84,254",
        delta: "+12.4%",
        up: true,
        spark: [
            14.0, 18.0, 16.0, 22.0, 20.0, 26.0, 24.0, 30.0, 28.0, 34.0, 38.0, 44.0,
        ],
    },
    Kpi {
        label: "Active users",
        value: "2,431",
        delta: "+4.1%",
        up: true,
        spark: [
            20.0, 22.0, 18.0, 24.0, 26.0, 22.0, 28.0, 30.0, 26.0, 32.0, 30.0, 34.0,
        ],
    },
    Kpi {
        label: "Conversion",
        value: "3.28%",
        delta: "-0.6%",
        up: false,
        spark: [
            36.0, 34.0, 38.0, 30.0, 32.0, 28.0, 30.0, 24.0, 26.0, 22.0, 20.0, 18.0,
        ],
    },
    Kpi {
        label: "Avg. order",
        value: "$94.12",
        delta: "+1.9%",
        up: true,
        spark: [
            24.0, 26.0, 22.0, 28.0, 24.0, 30.0, 28.0, 26.0, 32.0, 30.0, 34.0, 32.0,
        ],
    },
];

/// Returns the value text of metric card `k` at live tick `tick`. Tick 0 is the startup value;
/// every later tick gives another number of the same shape.
pub fn kpi_value(k: usize, tick: u64) -> String {
    if tick == 0 {
        return KPIS[k].value.to_owned();
    }
    let step = (tick * 37 + k as u64 * 11) % 97;
    match k {
        0 => format!("${}", thousands(84_000 + step * 13)),
        1 => thousands(2_400 + step),
        2 => format!("{}.{:02}%", 3, step),
        _ => format!("${}.{:02}", 90 + step % 9, step),
    }
}

/// Returns a number with a comma between every three digits.
fn thousands(value: u64) -> String {
    let mut text = value.to_string();
    let mut at = text.len();
    while at > 3 {
        at -= 3;
        text.insert(at, ',');
    }
    text
}

/// The height in px of chart bar `i` (0..24) of series `series` (0..3, `SEGMENTS`): between 40
/// and 216.
pub fn bar_height(series: usize, i: usize) -> f32 {
    let wave = [0, 22, 48, 70, 88, 96, 90, 74, 52, 30, 12, 4];
    let base = wave[(i + series * 3) % wave.len()] as f32;
    let jitter = ((i * 37 + series * 53) % 41) as f32;
    40.0 + base + jitter + 20.0 * series as f32
}

/// The height in px of chart bar `i` of the previous period of series `series`.
pub fn previous_height(series: usize, i: usize) -> f32 {
    (bar_height(series, (i + 5) % CHART_BARS) * 0.8).round()
}

/// The status of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Status {
    Paid,
    Refunded,
    Shipped,
    Pending,
}

impl Status {
    /// Every status in the order of the filter checkboxes.
    pub const ALL: [Status; 4] = [
        Status::Paid,
        Status::Pending,
        Status::Shipped,
        Status::Refunded,
    ];

    /// Returns the text of the pill and of the checkbox.
    pub fn label(self) -> &'static str {
        match self {
            Status::Paid => "Paid",
            Status::Refunded => "Refunded",
            Status::Shipped => "Shipped",
            Status::Pending => "Pending",
        }
    }

    /// Returns the pill background and text colors.
    pub fn colors(self) -> (u32, u32) {
        match self {
            Status::Paid => (0xdcfce7, 0x166534),
            Status::Refunded => (0xfee2e2, 0x991b1b),
            Status::Shipped => (0xdbeafe, 0x1e40af),
            Status::Pending => (0xfef3c7, 0x92400e),
        }
    }

    /// Returns the index of the status in `ALL`, which is its checkbox row.
    pub fn filter_row(self) -> usize {
        Status::ALL.iter().position(|s| *s == self).unwrap_or(0)
    }
}

/// The first names of the customers and authors.
pub const FIRST_NAMES: [&str; 16] = [
    "Olivia", "Liam", "Emma", "Noah", "Ava", "Elias", "Mia", "Lucas", "Sofia", "Mateo", "Clara",
    "Jonas", "Lea", "Felix", "Hanna", "Theo",
];
/// The last names of the customers and authors.
pub const LAST_NAMES: [&str; 13] = [
    "Schmidt", "Garcia", "Nguyen", "Okafor", "Rossi", "Kowalski", "Silva", "Tanaka", "Dubois",
    "Moreau", "Larsen", "Novak", "Petrov",
];

/// One order of the table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Order {
    /// The order number, shown as `#<number>`.
    pub number: u32,
    pub customer: String,
    /// The two initials of the customer.
    pub initials: String,
    pub avatar: u32,
    pub status: Status,
    /// The progress in percent, 0..=100.
    pub progress: u32,
    /// The amount in cents.
    pub cents: u64,
}

impl Order {
    /// Returns the amount as `$1,234.56`.
    pub fn amount(&self) -> String {
        format!("${}.{:02}", thousands(self.cents / 100), self.cents % 100)
    }

    /// Returns the order number as `#10200`.
    pub fn number_text(&self) -> String {
        format!("#{}", self.number)
    }
}

/// Returns order `i` of the table. Order 1 is refunded, so hiding refunded orders moves the rows
/// from row 1 on.
pub fn order(i: usize) -> Order {
    let first = FIRST_NAMES[(i * 7) % FIRST_NAMES.len()];
    let last = LAST_NAMES[(i * 5 + i / 3) % LAST_NAMES.len()];
    let status = [
        Status::Paid,
        Status::Refunded,
        Status::Shipped,
        Status::Pending,
    ][i % 4];
    Order {
        number: 10200 + i as u32,
        customer: format!("{first} {last}"),
        initials: format!("{}{}", &first[..1], &last[..1]),
        avatar: AVATAR_COLORS[(i * 5) % AVATAR_COLORS.len()],
        status,
        progress: ((i * 37) % 101) as u32,
        cents: cents(i),
    }
}

/// Returns the amount of order `i` in cents.
pub fn cents(i: usize) -> u64 {
    ((i as u64 * 7919) % 500_000) + 1_000
}

/// Returns the indices of the orders of the list in the order of its rows: by amount, largest
/// first, when `sorted`, and `0..count` otherwise. Orders with the same amount keep their order.
pub fn list_order(count: usize, sorted: bool) -> Vec<u32> {
    let mut indices: Vec<u32> = (0..count as u32).collect();
    if sorted {
        indices.sort_by_key(|i| std::cmp::Reverse(cents(*i as usize)));
    }
    indices
}

/// Returns the orders of a table of `count` rows.
pub fn orders(count: usize) -> Vec<Order> {
    (0..count).map(order).collect()
}

/// Returns true when an order passes the status filter (`shown[k]` for `Status::ALL[k]`) and the
/// search text, which matches the customer name without regard to case.
pub fn visible(order: &Order, shown: &[bool; 4], search: &str) -> bool {
    shown[order.status.filter_row()]
        && (search.is_empty()
            || order
                .customer
                .to_lowercase()
                .contains(&search.to_lowercase()))
}

/// Returns the orders in the order the table shows them: by amount, largest first, when
/// `sorted`, and as given otherwise. Orders with the same amount keep their order.
pub fn arranged(mut orders: Vec<Order>, sorted: bool) -> Vec<Order> {
    if sorted {
        orders.sort_by(|a, b| b.cents.cmp(&a.cents));
    }
    orders
}

/// The sentences of the message bodies.
pub const SENTENCES: [&str; 12] = [
    "The export for last quarter is ready and I attached the summary to the ticket.",
    "Could you take another look at the numbers for the northern region before the call?",
    "We shipped the fix this morning.",
    "Refunds are up a little this week, mostly from one campaign that ran with the wrong price.",
    "Thanks, that works for me.",
    "I moved the review to Thursday so that everyone from the warehouse team can join us.",
    "The new dashboard loads much faster on the old laptops, which the support team noticed at once.",
    "Please confirm the address.",
    "Two orders are still waiting for the customs papers and should go out early next week.",
    "Let me know if anything is missing.",
    "Conversion went down after the redesign of the checkout, so we are testing the old layout again.",
    "Great work on the launch!",
];

/// One message of the feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub author: String,
    /// The two initials of the author.
    pub initials: String,
    pub avatar: u32,
    /// The time, such as `09:41`.
    pub time: String,
    /// One to three sentences.
    pub body: String,
}

/// Returns message `i` of the feed.
pub fn message(i: usize) -> Message {
    let first = FIRST_NAMES[(i * 3 + 1) % FIRST_NAMES.len()];
    let last = LAST_NAMES[(i * 7 + 2) % LAST_NAMES.len()];
    let body = (0..=i % 3)
        .map(|k| SENTENCES[(i * 5 + k * 7) % SENTENCES.len()])
        .collect::<Vec<_>>()
        .join(" ");
    Message {
        author: format!("{first} {last}"),
        initials: format!("{}{}", &first[..1], &last[..1]),
        avatar: AVATAR_COLORS[(i * 3) % AVATAR_COLORS.len()],
        time: format!("{:02}:{:02}", 8 + (i / 6) % 10, (i * 7) % 60),
        body,
    }
}

/// Returns the messages of a feed of `count` cards.
pub fn messages(count: usize) -> Vec<Message> {
    (0..count).map(message).collect()
}

/// The number of elements the overview holds with `count` orders, and the messages page with
/// `count` messages, as the spec counts them: one per box, text, control and bar. Frameworks
/// build more or fewer nodes of their own; results use this count as the size of the app.
pub fn elements(count: usize) -> usize {
    CHROME_ELEMENTS + ORDER_ELEMENTS * count
}

/// The elements outside the orders: the sidebar (39), the top bar (8), the metric cards (69), the
/// chart card (84), the filter card (21), and the orders card with its header (7).
pub const CHROME_ELEMENTS: usize = 228;
/// The elements of one order row: the row, five cells, the avatar, the name, the pill, the track
/// and its fill.
pub const ORDER_ELEMENTS: usize = 11;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_geometry_adds_up() {
        let content = WINDOW.0 - SIDEBAR_WIDTH - 2.0 * CONTENT_PADDING;
        assert_eq!(content, 992.0);
        assert_eq!(KPI_WIDTH * 4.0 + GAP * 3.0, content);
        assert_eq!(CHART_WIDTH + GAP + FILTERS_WIDTH, content);
        assert_eq!(
            TOPBAR_HEIGHT + CONTENT_PADDING + KPI_HEIGHT + GAP + ROW2_HEIGHT + GAP,
            TABLE_TOP
        );
        assert_eq!(PLOT_X + CHART_SLOT * CHART_BARS as f32, 884.0);
        assert_eq!(SWITCH_X + SWITCH_WIDTH, 1236.0);
    }

    #[test]
    fn bars_fit_the_plot() {
        for s in 0..SEGMENTS.len() {
            for i in 0..CHART_BARS {
                let h = bar_height(s, i);
                assert!((40.0..=PLOT_BOTTOM - PLOT_TOP).contains(&h), "{s} {i} {h}");
                assert!(previous_height(s, i) <= PLOT_BOTTOM - PLOT_TOP);
            }
        }
    }

    #[test]
    fn orders_are_stable() {
        let o = order(1);
        assert_eq!(o.status, Status::Refunded);
        assert_eq!(o.number_text(), "#10201");
        assert_eq!(order(0).amount(), "$10.00");
        assert_eq!(order(130).amount(), "$304.70");
        assert_eq!(order(63).amount(), "$4,998.97");
    }

    #[test]
    fn filters_hide_and_search() {
        let o = order(1);
        assert!(visible(&o, &[true; 4], ""));
        assert!(!visible(&o, &[true, true, true, false], ""));
        let name = o.customer.to_uppercase();
        assert!(visible(&o, &[true; 4], &name[..3]));
        assert!(!visible(&o, &[true; 4], "zzz"));
    }

    #[test]
    fn sorting_puts_the_largest_amount_first() {
        let sorted = arranged(orders(50), true);
        assert!(sorted.windows(2).all(|w| w[0].cents >= w[1].cents));
        assert_ne!(sorted[0].number, order(0).number);
        assert_eq!(arranged(orders(50), false), orders(50));
    }

    #[test]
    fn the_list_sorts_like_the_table() {
        let by_amount: Vec<u32> = arranged(orders(200), true)
            .iter()
            .map(|o| o.number - 10200)
            .collect();
        assert_eq!(list_order(200, true), by_amount);
        assert_eq!(list_order(3, false), [0, 1, 2]);
    }

    #[test]
    fn live_values_start_at_the_startup_value() {
        for k in 0..KPI_COUNT {
            assert_eq!(kpi_value(k, 0), KPIS[k].value);
            assert_ne!(kpi_value(k, 1), kpi_value(k, 2));
        }
        assert_eq!(kpi_value(0, 1), "$84,481");
    }

    #[test]
    fn messages_are_stable() {
        let m = message(2);
        assert_eq!(m.body.split(". ").count(), 3);
        assert_eq!(message(0).time, "08:00");
        assert_eq!(messages(30).len(), 30);
        assert_eq!(page(MESSAGES_ITEM), Page::Messages);
        assert_eq!(page(0), Page::Overview);
        assert_eq!(page(3), Page::Empty);
    }

    #[test]
    fn the_new_controls_fit_the_top_bar() {
        assert!(SEARCH_X + SEARCH_WIDTH < LIVE_X);
        assert!(LIVE_X + SWITCH_WIDTH < SPINNER_X);
        assert!(SPINNER_X + SPINNER_SIZE < EXPORT_X);
        assert_eq!(MENU_X, EXPORT_X);
        assert!(MENU_X + MENU_WIDTH < WINDOW.0);
        assert_eq!(SORT_X + SORT_WIDTH, COL_AMOUNT_END);
    }
}
