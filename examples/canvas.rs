use rui::*;

fn main() {
    rui(canvas(|rect, vger| {
        vger.translate(rect.center() - LocalPoint::zero());

        let paint = vger.linear_gradient(
            [-100.0, -100.0].into(),
            [100.0, 100.0].into(),
            AZURE_HIGHLIGHT,
            RED_HIGHLIGHT,
            0.0,
        );

        let radius = 100.0;
        vger.fill_circle(LocalPoint::zero(), radius, paint);
    }));
}
