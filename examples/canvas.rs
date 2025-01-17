use rui::*;

fn main() {
    let mut data = Vec::new();
    let segments = 120;
    for i in 0..segments {
        let angle = i as f32 * std::f32::consts::PI / (segments as f32 / 2.0);
        let x = angle.cos() * 100.0;
        let y = angle.sin() * 100.0;
        data.push(LocalPoint::new(x, y));
    }

    canvas(move |_, rect, vger| {
        vger.save();

        vger.translate(rect.center() - LocalPoint::zero());

        let paint = vger.linear_gradient(
            [-100.0, -100.0],
            [100.0, 100.0],
            AZURE_HIGHLIGHT,
            RED_HIGHLIGHT,
            0.0,
        );

        for i in 0..segments {
            vger.stroke_segment(LocalPoint::zero(), data[i], 2.0, paint);
        }

        vger.restore();

        vger.fill_circle(LocalPoint::zero(), 50.0, paint);
    })
    .run()
}
