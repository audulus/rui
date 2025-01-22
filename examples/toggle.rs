use rui::*;

fn main() {
    state(
        || false,
        |s, _| {
            vstack((
                text("Toggle Example").font_size(40),
                toggle(s), // Default toggle
                Toggle::new().width(50.0).edge(3.0).show(s),
                Toggle::new().width(50.0).height(30.0).edge(3.0).show(s),
                Toggle::new().width(30.0).height(10.0).edge(3.0).show(s),
            ))
            .padding(Auto)
        },
    )
    .run()
}
