use rui::*;

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|_| println!("tapped circle"))
            .padding(Auto),
        state(LocalOffset::zero, |off, cx| {
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT.alpha(0.8))
                .offset(cx[off])
                .drag(move |cx, delta, _state| cx[off] += delta)
                .padding(Auto)
        }),
    )));
}
