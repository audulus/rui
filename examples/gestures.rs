use rui::*;

fn main() {
    rui(hstack! {
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|| { println!("tapped circle") })
            .padding(Auto);
        state(LocalOffset::zero(), |offset_state| {
            let off = offset_state.get();
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT.alpha(0.8))
                .offset(off)
                .drag(move |delta, _state| offset_state.set(offset_state.get() + delta) )
                .padding(Auto)
        })
    });
}
