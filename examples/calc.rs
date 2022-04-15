use rui::*;
use tao::keyboard::ModifiersState;

// XXX: WIP

fn digit_button(title: &str, state: State<String>) -> impl View {
    let t = String::from(title).clone();
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT)
            .tap(move |cx, _| cx[state].push_str(&t)),
        text(title),
    ))
    .padding(Auto)
}

fn calc_button(
    title: &str,
    callback: impl Fn(&mut Context, ModifiersState) + 'static,
) -> impl View {
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT)
            .tap(callback),
        text(title),
    ))
    .padding(Auto)
}

fn main() {
    rui(state(
        || String::from("0"),
        |s, cx| {
            vstack((
                text(&format!("{}", cx[s])),
                hstack((
                    calc_button("AC", move |cx, _| cx[s] = "0".into()),
                    calc_button("+/-", |_, _| ()),
                    calc_button("%", |_, _| ()),
                    calc_button("/", |_, _| ()),
                )),
                hstack((
                    digit_button("7", s),
                    digit_button("8", s),
                    digit_button("9", s),
                    calc_button("*", |_, _| ()),
                )),
                hstack((
                    digit_button("4", s),
                    digit_button("5", s),
                    digit_button("6", s),
                    calc_button("-", |_, _| ()),
                )),
                hstack((
                    digit_button("1", s),
                    digit_button("2", s),
                    digit_button("3", s),
                    calc_button("+", |_, _| ()),
                )),
                hstack((
                    digit_button("0", s),
                    calc_button(".", move |cx, _| cx[s].push('.')),
                    calc_button("=", |_, _| ()),
                )),
            ))
        },
    ))
}
