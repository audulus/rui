//! Headless profile of the update after a hover write: lays out the dashboard with vger's real
//! text measurement, then moves the pointer between two order rows and times each update.
//! Drawing isn't measured: it needs a window surface.

#[path = "../src/spec/cli.rs"]
mod cli;
#[path = "../src/spec/dashboard.rs"]
mod dash;
#[path = "../src/dashboard/mod.rs"]
#[allow(dead_code)]
mod dashboard;

use futures::executor::block_on;
use rui::*;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    let rows: usize = std::env::var("ROWS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100);
    let iters: usize = std::env::var("ITERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2000);

    let (device, queue) = block_on(async {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = wgpu::util::initialize_adapter_from_env_or_default(&instance, None)
            .await
            .expect("no adapter");
        adapter
            .request_device(&wgpu::DeviceDescriptor::default(), None)
            .await
            .expect("no device")
    });
    let mut vger = Vger::new(
        Arc::new(device),
        Arc::new(queue),
        wgpu::TextureFormat::Rgba8UnormSrgb,
    );

    let view = dashboard::view(rows);
    let mut cx = Context::new();
    let size = WorldRect::new(
        WorldPoint::origin(),
        [dash::WINDOW.0, dash::WINDOW.1].into(),
    )
    .size;
    let mut access = vec![];

    let t = Instant::now();
    cx.update(&view, &mut vger, &mut access, size);
    eprintln!("first update: {:.3} ms", t.elapsed().as_secs_f64() * 1e3);

    // Centers of the first two order rows, in rui's y-up window coordinates.
    let row_y = |k: f32| {
        dash::WINDOW.1
            - (dash::TABLE_TOP + dash::TABLE_HEADER_HEIGHT + dash::ORDER_ROW_HEIGHT * (k + 0.5))
    };
    let points = [
        LocalPoint::new(700.0, row_y(0.0)),
        LocalPoint::new(700.0, row_y(1.0)),
    ];

    let mut run = |label: &str, cx: &mut Context, write: u8| {
        let mut process = 0.0;
        let mut update = 0.0;
        let mut laid_out = 0;
        for i in 0..iters {
            if write == 2 {
                cx.invalidate_all();
            }
            if write == 1 {
                let event = Event::TouchMove {
                    id: 0,
                    position: points[i % 2],
                    delta: LocalOffset::zero(),
                };
                let t = Instant::now();
                cx.process(&view, &event);
                process += t.elapsed().as_secs_f64();
            }
            let t = Instant::now();
            if cx.update(&view, &mut vger, &mut access, size) {
                laid_out += 1;
            }
            update += t.elapsed().as_secs_f64();
        }
        eprintln!(
            "{label}: process {:.4} ms, update {:.4} ms per iteration ({laid_out}/{iters} updates did layout)",
            process * 1e3 / iters as f64,
            update * 1e3 / iters as f64
        );
    };
    let only = std::env::var("ONLY").ok();
    for (label, write) in [("idle", 0), ("hover", 1), ("invalidate_all", 2)] {
        if only.as_deref().is_none_or(|o| o == label) {
            run(label, &mut cx, write);
        }
    }
}
