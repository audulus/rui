//! The chart card: the title, the segmented control that picks the series, and the bar plot,
//! which a `canvas` paints. With compare on, every slot also holds the bar of the previous
//! period.

use rui::*;

use super::App;
use super::style::{Surface, color, gap, label};
use crate::dash;

/// The plot size: `CHART_BARS` slots across, from the top of the plot to its baseline.
const PLOT: [f32; 2] = [
    dash::CHART_SLOT * dash::CHART_BARS as f32,
    dash::PLOT_BOTTOM - dash::PLOT_TOP,
];
/// The top of the card in the window.
const CARD_TOP: f32 = dash::TABLE_TOP - dash::GAP - dash::ROW2_HEIGHT;
/// The top of the segmented control in the window.
const SEGMENTS_TOP: f32 = 232.0;

/// Returns the chart card, as wide as it is offered.
pub fn view(app: StateHandle<App>) -> impl View {
    memo(move |cx| {
        let series = cx[app].series;
        let compare = cx[app].compare;
        let track = 2.0;
        let header_height = dash::SEGMENT_HEIGHT + 2.0 * track;
        vstack((
            gap(0.0, SEGMENTS_TOP - CARD_TOP),
            hstack((
                gap(dash::CARD_PADDING, header_height),
                label(dash::CHART_TITLE, dash::TITLE_FONT, dash::TEXT),
                spacer(),
                segments(app, series, track),
                gap(dash::CARD_PADDING, 0.0),
            )),
            gap(0.0, dash::PLOT_TOP - SEGMENTS_TOP - header_height),
            hstack((
                gap(dash::CARD_PADDING, 0.0),
                plot(series, compare),
                spacer(),
            )),
            gap(0.0, dash::CARD_PADDING),
        ))
        .background(Surface::CARD.view())
    })
}

/// The segmented control with series `selected` selected.
fn segments(app: StateHandle<App>, selected: usize, track: f32) -> impl View {
    let segment = move |i: usize| {
        let on = i == selected;
        let text = if on { dash::TEXT } else { dash::MUTED };
        let surface = Surface {
            fill: on.then_some(0xffffff),
            radius: 6.0,
            border: None,
            shadow: on.then_some(dash::SHADOW_SMALL),
        };
        label(dash::SEGMENTS[i], 13.0, text)
            .fixed_width(dash::SEGMENT_WIDTH)
            .fixed_height(dash::SEGMENT_HEIGHT)
            .background(surface.view())
            .tap(move |cx| {
                if cx[app].series != i {
                    cx[app].series = i;
                }
            })
    };
    hstack((
        gap(track, dash::SEGMENT_HEIGHT + 2.0 * track),
        segment(0),
        segment(1),
        segment(2),
        gap(track, 0.0),
    ))
    .background(Surface::fill(dash::SEGMENT_TRACK, 8.0).view())
}

/// The plot of `series`, with the previous period when `compare` is on.
fn plot(series: usize, compare: bool) -> impl View {
    canvas(move |_, rect, vger| {
        let grid = vger.color_paint(color(dash::GRID));
        for y in dash::GRID_LINES {
            // The line's top, measured up from the baseline.
            let up = dash::PLOT_BOTTOM - y - 1.0;
            vger.fill_rect(
                LocalRect::new([0.0, up].into(), [rect.width(), 1.0].into()),
                0.0,
                grid,
            );
        }
        let bar = vger.color_paint(color(dash::BAR));
        let previous = vger.color_paint(color(dash::BAR_PREVIOUS));
        for i in 0..dash::CHART_BARS {
            let slot = i as f32 * dash::CHART_SLOT;
            let current = dash::bar_height(series, i);
            if compare {
                let old = dash::previous_height(series, i);
                vger.fill_rect(
                    LocalRect::new([slot + 3.0, 0.0].into(), [8.0, old].into()),
                    4.0,
                    previous,
                );
                vger.fill_rect(
                    LocalRect::new([slot + 13.0, 0.0].into(), [8.0, current].into()),
                    4.0,
                    bar,
                );
            } else {
                vger.fill_rect(
                    LocalRect::new([slot + 4.0, 0.0].into(), [17.0, current].into()),
                    4.0,
                    bar,
                );
            }
        }
    })
    .size(PLOT)
}
