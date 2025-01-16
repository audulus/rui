use rui::*;

fn anim_to(current: &mut LocalOffset, target: LocalOffset) -> bool {
    if *current != target {
        if (*current - target).length() < 0.01 {
            *current = target;
        } else {
            *current = current.lerp(target, 0.03);
        }
        true
    } else {
        false
    }
}

#[derive(Default)]
struct MyState {
    animated: LocalOffset,
    dragged: LocalOffset,
}

fn main() {
    hstack((
        circle()
            .color(RED_HIGHLIGHT.alpha(0.8))
            .tap(|_cx| println!("tapped circle"))
            .padding(Auto),
        // target offset
        state(MyState::default, move |s, cx| {
            // animated offset
            rectangle()
                .corner_radius(5.0)
                .color(AZURE_HIGHLIGHT.alpha(0.8))
                .offset(cx[s].animated)
                .drag(move |cx, delta, state, _| {
                    cx[s].dragged += delta;
                    cx[s].animated = cx[s].dragged;
                    if state == GestureState::Ended {
                        cx[s].dragged = LocalOffset::zero();
                    }
                })
                .anim(move |cx, _dt| {
                    let mut v = cx[s].animated;
                    if anim_to(&mut v, cx[s].dragged) {
                        cx[s].animated = v;
                    }
                })
                .padding(Auto)
        }),
    ))
    .run()
}
