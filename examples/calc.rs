use rui::*;

fn digit_button(title: &str, binding: impl Binding<String>) -> impl View {
    let t = String::from(title).clone();
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT)
            .tap(move |cx| binding.with_mut(cx, |value| value.push_str(&t))),
        text(title)
    )).padding(Auto)
}

fn calc_button(title: &str, callback: impl Fn(&mut Context) + 'static) -> impl View {
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT)
            .tap(callback),
        text(title)
    )).padding(Auto)
}

fn main() {
    rui(
        state(|| String::from("0"),
              |s, cx|
                vstack((
                    text(&format!("{}",cx[s])),
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
                        calc_button(".", move |cx| cx[s].push('.')),
                        calc_button("=", |_| ()),
                    ))
                ))
        )
    )
}