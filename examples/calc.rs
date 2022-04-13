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
                        calc_button("7", move |cx| cx[s].push('7')),
                        calc_button("8", move |cx| cx[s].push('8')),
                        calc_button("9", move |cx| cx[s].push('9')),
                        calc_button("*", |_| ()),
                    )),
                    hstack((
                        calc_button("4", move |cx| cx[s].push('4')),
                        calc_button("5", move |cx| cx[s].push('5')),
                        calc_button("6", move |cx| cx[s].push('6')),
                        calc_button("-", |_| ()),
                    )),
                    hstack((
                        calc_button("1", move |cx| cx[s].push('1')),
                        calc_button("2", move |cx| cx[s].push('2')),
                        calc_button("3", move |cx| cx[s].push('3')),
                        calc_button("+", |_| ()),
                    )),
                    hstack((
                        calc_button("0", move |cx| cx[s].push('0')),
                        calc_button(".", move |cx| cx[s].push('.')),
                        calc_button("=", |_| ()),
                    ))
                ))
        )
    )
}