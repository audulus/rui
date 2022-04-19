use rui::*;

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|_cx, _key_mods| println!("tapped circle"))
            .padding(Auto),
        state(LocalOffset::zero, move |off, _| { // target offset
            state(LocalOffset::zero, move |anim_off, cx| { // animated offset
                rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT.alpha(0.8))
                .offset(cx[anim_off])
                .drag(move |cx, delta, state, _, _| {
                    cx[off] += delta;
                    cx[anim_off] = cx[off];
                    if state == GestureState::Ended {
                        cx[off] = LocalOffset::zero();
                    }
                })
                .anim(move |cx, _dt| {
                    if cx[anim_off] != cx[off] {
                        if (cx[anim_off] - cx[off]).length() < 0.01 {
                            cx[anim_off] = cx[off];
                        } else {
                            cx[anim_off] = cx[anim_off].lerp(cx[off], 0.05);
                        }
                    }
                })
                .padding(Auto)
            })
        }),
    )));
}
