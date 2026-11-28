//! The live updates: while the live switch is on, a tick every `LIVE_INTERVAL_MS` gives the
//! metric cards new values, and the spinner turns.
//!
//! rui has no timer. As its `async` example teaches for background work, a thread sleeps and
//! posts each tick to the UI with `on_main`. The spinner is a square on a `canvas` that rui
//! turns with vger; an `anim` callback marks it changed on every frame while it shows.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use rui::*;

use super::App;
use super::style::{color, gap};
use crate::dash;

/// The state of the live updates.
#[derive(Default)]
pub struct Live {
    /// The live switch.
    pub on: bool,
    /// The ticks since the switch went on; 0 shows the startup values.
    pub tick: u64,
    /// Tells the thread of the current run to stop.
    running: Option<Arc<AtomicBool>>,
}

/// Turns the live updates on or off.
pub fn set(app: StateHandle<App>, on: bool, cx: &mut Context) {
    if cx[app].live.on == on {
        return;
    }
    let live = &mut cx[app].live;
    live.on = on;
    live.tick = 0;
    if let Some(running) = live.running.take() {
        running.store(false, Ordering::Relaxed);
    }
    if on {
        let running = Arc::new(AtomicBool::new(true));
        live.running = Some(running.clone());
        thread::spawn(move || {
            let interval = Duration::from_millis(dash::LIVE_INTERVAL_MS);
            let mut next = Instant::now() + interval;
            loop {
                thread::sleep(next.saturating_duration_since(Instant::now()));
                next += interval;
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                let running = running.clone();
                on_main(move |cx| {
                    if running.load(Ordering::Relaxed) {
                        cx[app].live.tick += 1;
                    }
                });
            }
        });
    }
}

/// The spinner slot of the top bar: the turning square while live updates are on.
pub fn spinner(app: StateHandle<App>) -> impl View {
    memo(move |cx| {
        cond(
            cx[app].live.on,
            any_view(turning()),
            any_view(gap(dash::SPINNER_SIZE, dash::SPINNER_SIZE)),
        )
    })
}

/// The square, turning once every `SPINNER_PERIOD_MS` at a constant speed from when it shows.
fn turning() -> impl View {
    state(
        || (Instant::now(), 0u64),
        move |frame, _| {
            canvas(move |cx, rect, vger| {
                let period = dash::SPINNER_PERIOD_MS as f32 / 1000.0;
                let turns = cx[frame].0.elapsed().as_secs_f32() / period;
                let half = dash::SPINNER_SIZE / 2.0;
                let paint = vger.color_paint(color(dash::PRIMARY));
                vger.save();
                vger.translate(rect.center().to_vector());
                vger.rotate(-turns.fract() * std::f32::consts::TAU);
                vger.fill_rect(
                    LocalRect::new([-half, -half].into(), [dash::SPINNER_SIZE; 2].into()),
                    dash::SPINNER_RADIUS,
                    paint,
                );
                vger.restore();
            })
            .size([dash::SPINNER_SIZE, dash::SPINNER_SIZE])
            .anim(move |cx, _| cx[frame].1 += 1)
        },
    )
}
