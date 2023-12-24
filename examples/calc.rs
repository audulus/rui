use rui::*;

// XXX: WIP

fn digit_button(title: &str, state: StateHandle<String>) -> impl View {
    let t = String::from(title);
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT)
            .tap(move |cx| cx[state].push_str(&t)),
        text(title).color(BLACK).offset([10.0, 10.0]),
    ))
    .padding(Auto)
}

fn calc_button(title: &str, callback: impl Fn(&mut Context) + 'static) -> impl View {
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(GREEN_HIGHLIGHT)
            .tap(callback),
        text(title).color(BLACK).offset([10.0, 10.0]),
    ))
    .padding(Auto)
}

fn main() {
    state(
        || String::from("0"),
        |s, cx| {
            vstack((
                text(&cx[s].to_string()),
                hstack((
                    calc_button("AC", move |cx| cx[s] = "0".into()),
                    calc_button("+/-", |_| ()),
                    calc_button("%", |_| ()),
                    calc_button("/", |_| ()),
                )),
                hstack((
                    digit_button("7", s),
                    digit_button("8", s),
                    digit_button("9", s),
                    calc_button("*", |_| ()),
                )),
                hstack((
                    digit_button("4", s),
                    digit_button("5", s),
                    digit_button("6", s),
                    calc_button("-", |_| ()),
                )),
                hstack((
                    digit_button("1", s),
                    digit_button("2", s),
                    digit_button("3", s),
                    calc_button("+", |_| ()),
                )),
                hstack((
                    digit_button("0", s),
                    calc_button(".", move |cx| cx[s].push_str(".")),
                    calc_button("=", |_| ()),
                )),
            ))
        },
    )
    .run()
}
