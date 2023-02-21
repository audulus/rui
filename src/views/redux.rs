use crate::*;

pub fn redux<
    V: View,
    A: 'static,
    S: 'static,
    D: Fn() -> S + 'static,
    F: Fn(&S) -> V + 'static,
    R: Fn(&mut S, &A) + 'static + Clone,
>(
    initial: D,
    f: F,
    reducer: R,
) -> impl View {
    state(initial, move |state_handle, cx| {
        let r = reducer.clone();
        f(&cx[state_handle]).handle(move |cx, action: &A| r(&mut cx[state_handle], action))
    })
}
