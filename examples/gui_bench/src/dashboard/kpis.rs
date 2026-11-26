//! The four metric cards: label, value, delta pill and sparkline. While live updates are on, the
//! values follow the tick.

use rui::*;

use super::App;
use super::style::{Surface, color, gap, label, leading, pill};
use crate::dash;

/// Returns the row of metric cards, which share the width.
pub fn view(app: StateHandle<App>) -> impl View {
    state(
        || (),
        move |_, cx| {
            let tick = cx[app].live.tick;
            let card = |k: usize| card(k, dash::kpi_value(k, tick)).flex();
            hstack((
                card(0),
                gap(dash::GAP, 0.0),
                card(1),
                gap(dash::GAP, 0.0),
                card(2),
                gap(dash::GAP, 0.0),
                card(3),
            ))
        },
    )
}

/// Metric card `k` showing `value`.
fn card(k: usize, value: String) -> impl View {
    let kpi = dash::KPIS[k];
    let (bg, fg) = if kpi.up {
        (dash::UP_BG, dash::UP_TEXT)
    } else {
        (dash::DOWN_BG, dash::DOWN_TEXT)
    };
    let texts = vstack((
        gap(0.0, 18.0),
        leading(label(kpi.label, 13.0, dash::MUTED)).fixed_height(16.0),
        gap(0.0, 6.0),
        leading(label(&value, 24.0, dash::TEXT)).fixed_height(28.0),
        gap(0.0, 10.0),
        leading(pill(kpi.delta, 12.0, fg, 22.0, 8.0, bg)),
        gap(0.0, dash::KPI_HEIGHT - 100.0),
    ));
    hstack((
        gap(dash::CARD_PADDING, 0.0),
        texts.flex(),
        sparkline(kpi.spark),
        gap(dash::CARD_PADDING, 0.0),
    ))
    .fixed_height(dash::KPI_HEIGHT)
    .background(Surface::CARD.view())
}

/// The sparkline: bars with their bottoms on card y 92, as tall as the card.
fn sparkline(heights: [f32; dash::SPARK_BARS]) -> impl View {
    let width = dash::SPARK_STEP * (dash::SPARK_BARS - 1) as f32 + dash::SPARK_BAR_WIDTH;
    canvas(move |_, rect, vger| {
        let paint = vger.color_paint(color(dash::SPARK));
        let bottom = rect.height() - 92.0;
        for (i, h) in heights.iter().enumerate() {
            let x = i as f32 * dash::SPARK_STEP;
            vger.fill_rect(
                LocalRect::new([x, bottom].into(), [dash::SPARK_BAR_WIDTH, *h].into()),
                2.0,
                paint,
            );
        }
    })
    .size([width, dash::KPI_HEIGHT])
}
