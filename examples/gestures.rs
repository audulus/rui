use rui::*;

fn main() {
    rui(hstack! {
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|| { println!("tapped circle") })
            .padding(Auto);
        state(LocalOffset::zero(), |offset_state: State<LocalOffset>| {
            let off = offset_state.get();
            rectangle(5.0)
                .color(AZURE_HIGHLIGHT.alpha(0.8))
                .offset(off)
                .drag(move |off, state| offset_state.set(off) )
                .padding(Auto)
        })
    });
}
