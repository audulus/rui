use rui::*;

fn anim_to(current: &mut LocalOffset, target: LocalOffset) -> bool {
    if *current != target {
        if (*current - target).length() < 0.01 {
            *current = target;
        } else {
            *current = current.lerp(target, 0.05);
        }
        true
    } else {
        false
    }
}

fn main() {
    rui(hstack((
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|_cx, _key_mods| println!("tapped circle"))
            .padding(Auto),
        state(LocalOffset::zero, move |off, _| {
            // target offset
            state(LocalOffset::zero, move |anim_off, cx| {
                // animated offset
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
                        let mut v = cx[anim_off];
                        if anim_to(&mut v, cx[off]) {
                            cx[anim_off] = v;
                        }
                    })
                    .padding(Auto)
            })
        }),
    )));
}
