//! Tests of the node tree that stores per-view data.

use crate::*;
use std::cell::Cell;
use std::rc::Rc;

fn layout(cx: &mut Context, view: &impl View) {
    let root = cx.root_id();
    view.layout(
        root,
        &mut LayoutArgs {
            sz: [200.0, 200.0].into(),
            cx,
            text_bounds: &mut |_, _, _| LocalRect::zero(),
        },
    );
}

fn counter_list(ids: Vec<i32>) -> impl View {
    list(ids, |_| {
        state(|| 0i32, |_, _| rectangle().size([10.0, 10.0]))
    })
}

#[test]
fn test_list_state_follows_its_key() {
    let mut cx = Context::new();
    layout(&mut cx, &counter_list(vec![1, 2, 3]));

    let id = cx.id_at(&[0, hh(&2)]);
    let s = StateHandle::<i32>::new(id);
    cx[s] = 5;

    layout(&mut cx, &counter_list(vec![3, 1, 2]));
    assert_eq!(cx.id_at(&[0, hh(&2)]), id);
    assert_eq!(cx[s], 5);
}

#[test]
fn test_list_frees_removed_keys() {
    let mut cx = Context::new();
    let before = cx.live_nodes();
    layout(&mut cx, &counter_list(vec![1, 2, 3]));
    let id = cx.id_at(&[0, hh(&2)]);

    layout(&mut cx, &counter_list(vec![1, 3]));
    assert!(!cx.is_live(id));

    layout(&mut cx, &counter_list(vec![]));
    assert_eq!(cx.live_nodes(), before);
}

#[test]
fn test_list_duplicate_keys_share_a_node() {
    let mut cx = Context::new();
    let before = cx.live_nodes();
    layout(&mut cx, &counter_list(vec![1, 1, 2]));
    layout(&mut cx, &counter_list(vec![2]));
    layout(&mut cx, &counter_list(vec![]));
    assert_eq!(cx.live_nodes(), before);
}

#[test]
fn test_freed_slots_are_reused_with_a_new_generation() {
    let mut cx = Context::new();
    layout(&mut cx, &counter_list(vec![1]));
    let old = cx.id_at(&[0, hh(&1)]);
    layout(&mut cx, &counter_list(vec![]));
    layout(&mut cx, &counter_list(vec![2]));
    let new = cx.id_at(&[0, hh(&2)]);
    assert!(!cx.is_live(old));
    assert!(cx.is_live(new));
    assert_ne!(old, new);
}

#[test]
#[should_panic(expected = "no longer exists")]
fn test_stale_handle_panics() {
    let mut cx = Context::new();
    layout(&mut cx, &counter_list(vec![1]));
    let s = StateHandle::<i32>::new(cx.id_at(&[0, hh(&1)]));
    layout(&mut cx, &counter_list(vec![]));
    // Reuse the slot, so a missing generation check would read the new state.
    layout(&mut cx, &counter_list(vec![2]));
    let _ = cx[s];
}

fn switch(flag: bool) -> impl View {
    cond(
        flag,
        state(|| 1i32, |_, _| rectangle()),
        state(|| 2i32, |_, _| circle()),
    )
}

#[test]
fn test_cond_frees_the_hidden_branch() {
    let mut cx = Context::new();
    layout(&mut cx, &switch(true));
    let a = cx.id_at(&[0, 0]);
    cx[StateHandle::<i32>::new(a)] = 10;

    layout(&mut cx, &switch(false));
    assert!(!cx.is_live(a));
    let b = cx.id_at(&[0, 1]);
    assert_eq!(cx[StateHandle::<i32>::new(b)], 2);

    // Flipping back starts the first branch over.
    layout(&mut cx, &switch(true));
    assert!(!cx.is_live(b));
    assert_eq!(cx[StateHandle::<i32>::new(cx.id_at(&[0, 0]))], 1);
}

fn either(flag: bool) -> AnyView {
    if flag {
        any_view(state(|| 7i32, |_, _| rectangle()))
    } else {
        any_view(state(|| String::from("x"), |_, _| circle()))
    }
}

#[test]
fn test_any_view_type_change_resets_state() {
    let mut cx = Context::new();
    layout(&mut cx, &either(true));
    let first = cx.root_id();
    let child = |cx: &Context| {
        let entries = cx.layout_entries();
        // The root's only child: [0, hash of the type].
        cx.id_at(&entries.iter().find(|(p, _)| p.len() == 2).unwrap().0)
    };
    let a = child(&cx);
    cx[StateHandle::<i32>::new(a)] = 9;

    // Another type in the same slot gets fresh state instead of panicking on the downcast.
    layout(&mut cx, &either(false));
    assert!(!cx.is_live(a));
    assert_eq!(cx[StateHandle::<String>::new(child(&cx))], "x");

    layout(&mut cx, &either(true));
    assert_eq!(cx[StateHandle::<i32>::new(child(&cx))], 7);
    assert_eq!(cx.root_id(), first);
}

#[test]
fn test_state_layout_is_cached_until_state_changes() {
    let mut cx = Context::new();
    let ui = state(|| 10.0f32, |w, cx| rectangle().size([cx[w], 10.0]));
    layout(&mut cx, &ui);
    let s = StateHandle::<f32>::new(cx.root_id());
    assert_eq!(cx.get_layout(cx.root_id()).rect.width(), 10.0);

    // Writing the state invalidates the cached layout.
    cx[s] = 30.0;
    layout(&mut cx, &ui);
    assert_eq!(cx.get_layout(cx.root_id()).rect.width(), 30.0);
    cx.clear_dirty();

    // A layout epoch change, as on a window resize, also does.
    *cx.get_mut(s) = 40.0;
    cx.clear_dirty();
    layout(&mut cx, &ui);
    assert_eq!(cx.get_layout(cx.root_id()).rect.width(), 30.0);
    cx.layout_epoch += 1;
    layout(&mut cx, &ui);
    assert_eq!(cx.get_layout(cx.root_id()).rect.width(), 40.0);
}

/// Rows of `ids`, 20 tall, that count the drags each one sees.
fn draggable_rows(ids: Vec<i32>, drags: Rc<[Cell<usize>; 3]>) -> impl View {
    list(ids, move |&i| {
        let drags = drags.clone();
        rectangle()
            .size([50.0, 20.0])
            .drag(move |_, _, _, _| drags[i as usize].set(drags[i as usize].get() + 1))
    })
}

#[test]
fn test_removing_a_dragged_view_releases_its_touch() {
    let drags = Rc::new([Cell::new(0), Cell::new(0), Cell::new(0)]);
    let mut cx = Context::new();
    let root = cx.root_id();
    let mut actions = vec![];

    // Two rows: row 1 at y 20..40, row 2 at y 0..20.
    let ui = draggable_rows(vec![1, 2], drags.clone());
    layout(&mut cx, &ui);
    let begin = |y: f32| Event::TouchBegin {
        id: 0,
        position: [25.0, y].into(),
    };
    ui.process(&begin(30.0), root, &mut cx, &mut actions);
    assert_eq!(drags[1].get(), 1);
    assert!(!cx.touches[0].is_default());

    // Row 1 goes away mid-drag, so its touch is released.
    let ui = draggable_rows(vec![2], drags.clone());
    layout(&mut cx, &ui);
    assert!(cx.touches[0].is_default());

    // A new drag on row 2 starts.
    ui.process(&begin(10.0), root, &mut cx, &mut actions);
    assert_eq!(drags[2].get(), 1);
}

#[test]
fn test_removing_the_focused_view_clears_the_focus() {
    let mut cx = Context::new();
    let ui = counter_list(vec![1]);
    layout(&mut cx, &ui);
    cx.focused_id = Some(cx.id_at(&[0, hh(&1)]));
    layout(&mut cx, &counter_list(vec![]));
    assert_eq!(cx.focused_id, None);
}
