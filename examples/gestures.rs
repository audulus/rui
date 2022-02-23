use rui::*;

fn main() {
    rui(hstack! {
        circle()
            .color(RED_HIGHLIGHT)
            .tap(|| { println!("tapped circle") })
            .padding(Auto);
        state(LocalOffset::zero(), |offset_state: State<LocalOffset>| {
            let off = *offset_state.get();
            rectangle(5.0)
                .color(AZURE_HIGHLIGHT)
                .offset(off)
                .drag(move |off| *offset_state.get() = off )
                .padding(Auto)
        })
    });
}
