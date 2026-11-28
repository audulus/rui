//! Tests of the node tree that stores per-view data.

use crate::*;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::Arc;

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
fn test_list_keys_are_matched_again_after_a_write() {
    let mut cx = Context::new();
    // A list in a `state` view, whose state stands for what the app builds the keys from.
    let ui = |ids: Vec<i32>| {
        state(
            || 0i32,
            move |_, _| {
                list_shared(Arc::new(ids.clone()), |_| {
                    state(|| 0i32, |_, _| rectangle().size([10.0, 10.0]))
                })
            },
        )
    };
    layout(&mut cx, &ui(vec![1, 2, 3]));
    let outer = StateHandle::<i32>::new(cx.root_id());
    let list_id = cx.child(cx.root_id(), 0);
    let id = cx.id_at(&[0, 0, hh(&2)]);
    let item = StateHandle::<i32>::new(id);

    // Nothing was written, so the list doesn't look at its keys: views are built from the
    // states, and keys that change otherwise need `invalidate_all`. Key 2's node stays in slot 1.
    layout(&mut cx, &ui(vec![2, 1, 3]));
    assert_eq!(cx.child(list_id, 1), id);

    // Nor after a write to an item's own state: the views around a list don't read those.
    cx[item] = 5;
    layout(&mut cx, &ui(vec![2, 1, 3]));
    assert_eq!(cx.child(list_id, 1), id);

    // After any other write it matches them again; the node of key 2 moves with it.
    cx[outer] = 1;
    layout(&mut cx, &ui(vec![2, 3, 1]));
    assert_eq!(cx.child(list_id, 0), id);
    assert_eq!(cx[item], 5);

    // `invalidate_all` does too.
    cx.invalidate_all();
    layout(&mut cx, &ui(vec![1, 3, 2]));
    assert_eq!(cx.child(list_id, 2), id);
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

/// Counts calls per item: of the list's item function, and of each item's `state` function.
#[derive(Clone, Default)]
struct Calls {
    item: Rc<[Cell<usize>; 3]>,
    body: Rc<[Cell<usize>; 3]>,
}

impl Calls {
    fn items(&self) -> [usize; 3] {
        [0, 1, 2].map(|i| self.item[i].get())
    }
    fn bodies(&self) -> [usize; 3] {
        [0, 1, 2].map(|i| self.body[i].get())
    }
    fn reset(&self) {
        for i in 0..3 {
            self.item[i].set(0);
            self.body[i].set(0);
        }
    }
}

/// Three rows, each a `state` with a width, inside an outer `state` holding a spacing.
fn rows(calls: &Calls) -> impl View + use<> {
    let calls = calls.clone();
    state(
        || 0.0f32,
        move |spacing, cx| {
            let spacing = cx[spacing];
            let calls = calls.clone();
            list(vec![0usize, 1, 2], move |&i| {
                calls.item[i].set(calls.item[i].get() + 1);
                let calls = calls.clone();
                state(
                    || 10.0f32,
                    move |w, cx| {
                        calls.body[i].set(calls.body[i].get() + 1);
                        rectangle()
                            .size([cx[w], 10.0 + spacing])
                            .role(accesskit::Role::Button)
                    },
                )
            })
        },
    )
}

fn row_handle(cx: &Context, i: usize) -> StateHandle<f32> {
    StateHandle::new(cx.id_at(&[0, 0, hh(&i)]))
}

#[test]
fn test_layout_skips_clean_rows() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    assert_eq!(calls.bodies(), [1, 1, 1]);

    // Nothing changed: no row is rebuilt.
    calls.reset();
    layout(&mut cx, &ui);
    assert_eq!(calls.items(), [0, 0, 0]);
    assert_eq!(calls.bodies(), [0, 0, 0]);

    // A write to row 1 rebuilds row 1 only, and its new size is used.
    let s = row_handle(&cx, 1);
    cx[s] = 30.0;
    calls.reset();
    layout(&mut cx, &ui);
    assert_eq!(calls.items(), [0, 1, 0]);
    assert_eq!(calls.bodies(), [0, 1, 0]);
    assert_eq!(
        cx.get_layout(cx.id_at(&[0, 0, hh(&1usize)])).rect.width(),
        30.0
    );
}

#[test]
fn test_layout_redoes_rows_when_an_enclosing_state_changes() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);

    let spacing = StateHandle::<f32>::new(cx.root_id());
    cx[spacing] = 5.0;
    calls.reset();
    layout(&mut cx, &ui);
    assert_eq!(calls.bodies(), [1, 1, 1]);
    assert_eq!(
        cx.get_layout(cx.id_at(&[0, 0, hh(&0usize)])).rect.height(),
        15.0
    );
}

#[test]
fn test_write_then_clear_still_invalidates() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);

    // The write is cleared before any layout sees it; the cache must still go stale.
    let s = row_handle(&cx, 2);
    cx[s] = 20.0;
    cx.clear_dirty();
    calls.reset();
    layout(&mut cx, &ui);
    assert_eq!(calls.bodies(), [0, 0, 1]);
}

#[test]
fn test_invalidate_all_redoes_everything() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);

    cx.invalidate_all();
    calls.reset();
    layout(&mut cx, &ui);
    assert_eq!(calls.bodies(), [1, 1, 1]);
}

#[test]
fn test_layout_redoes_a_row_given_another_size() {
    let mut cx = Context::new();
    let ui = state(|| (), |_, _| rectangle());
    let root = cx.root_id();
    let lay = |cx: &mut Context, sz: LocalSize| {
        ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        )
    };
    assert_eq!(lay(&mut cx, [50.0, 50.0].into()), [50.0, 50.0].into());
    assert_eq!(lay(&mut cx, [80.0, 20.0].into()), [80.0, 20.0].into());
}

#[test]
fn test_access_sends_only_changed_rows() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();

    let mut nodes = vec![];
    ui.access(root, &mut cx, &mut nodes);
    let all: Vec<_> = nodes.iter().map(|(id, _)| *id).collect();
    // The list and three buttons.
    assert_eq!(all.len(), 4);
    cx.clear_dirty();

    // Nothing changed: nothing is sent.
    nodes.clear();
    ui.access(root, &mut cx, &mut nodes);
    assert!(nodes.is_empty());

    // A write to row 0 sends the list, whose subtree changed, and row 0's node.
    let s = row_handle(&cx, 0);
    cx[s] = 12.0;
    calls.reset();
    nodes.clear();
    ui.access(root, &mut cx, &mut nodes);
    assert_eq!(nodes.len(), 2);
    assert_eq!(calls.bodies(), [1, 0, 0]);
    assert!(nodes.iter().all(|(id, _)| all.contains(id)));
}

#[test]
fn test_access_tree_is_built_only_when_enabled() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let mut nodes = vec![];

    // Off by default: nothing is built, and no row's view is built for it.
    assert!(!cx.access_enabled());
    calls.reset();
    cx.update_access(&ui, &mut nodes);
    assert!(nodes.is_empty());
    assert_eq!(calls.bodies(), [0, 0, 0]);
    cx.clear_dirty();

    // Turning it on marks the UI dirty and the next update gives the whole tree: the list and
    // three buttons.
    cx.set_access_enabled(true);
    assert!(cx.dirty);
    cx.update_access(&ui, &mut nodes);
    assert_eq!(nodes.len(), 4);
    cx.clear_dirty();
    cx.update_access(&ui, &mut nodes);
    assert!(nodes.is_empty());

    // Off again, then on with nothing written in between: the caches from before must not
    // report the nodes as already sent.
    cx.set_access_enabled(false);
    assert!(!cx.dirty);
    cx.update_access(&ui, &mut nodes);
    assert!(nodes.is_empty());
    cx.set_access_enabled(true);
    cx.update_access(&ui, &mut nodes);
    assert_eq!(nodes.len(), 4);
}

#[test]
fn test_dirty_regions_skip_clean_rows() {
    let calls = Calls::default();
    let ui = rows(&calls);
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    cx.clear_dirty();
    let root = cx.root_id();

    let s = row_handle(&cx, 2);
    cx[s] = 40.0;
    calls.reset();
    ui.dirty(root, LocalToWorld::identity(), &mut cx);
    assert_eq!(calls.items(), [0, 0, 1]);
    assert_eq!(calls.bodies(), [0, 0, 0]);
    assert_eq!(cx.dirty_region.rects().len(), 1);
}

#[test]
fn test_anim_pass_visits_only_anim_subtrees() {
    let ticks = Rc::new(Cell::new(0));
    let built = Rc::new([Cell::new(0), Cell::new(0)]);
    let (t, b) = (ticks.clone(), built.clone());
    let ui = list(vec![0usize, 1], move |&i| {
        b[i].set(b[i].get() + 1);
        let t = t.clone();
        state(
            || 0,
            move |_, _| {
                let t = t.clone();
                cond(
                    i == 0,
                    rectangle().anim(move |_, _| t.set(t.get() + 1)),
                    rectangle(),
                )
            },
        )
    });
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();
    built[0].set(0);
    built[1].set(0);

    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(ticks.get(), 1);
    assert_eq!([built[0].get(), built[1].get()], [1, 0]);
}

/// Lays `view` out as an update does, after which the `has_anim` marks are complete.
fn update_layout(cx: &mut Context, view: &impl View) {
    layout(cx, view);
    cx.marked_gen = Some(cx.write_gen);
}

#[test]
fn test_anim_pass_skips_lists_without_anim() {
    let built = Rc::new(Cell::new(0));
    let b = built.clone();
    let ui = list(vec![0usize, 1, 2], move |_| {
        b.set(b.get() + 1);
        rectangle().size([10.0, 10.0])
    });
    let mut cx = Context::new();
    let root = cx.root_id();

    // Before the list's first layout the marks may be missing, so the pass visits the items.
    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(built.get(), 3);

    update_layout(&mut cx, &ui);
    built.set(0);
    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(built.get(), 0);

    // After a write the tree may hold new `anim` views until the next layout.
    cx.invalidate_all();
    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(built.get(), 3);
}

#[test]
fn test_anim_pass_reaches_anim_in_plain_list_items() {
    let ticks = Rc::new(Cell::new(0));
    let built = Rc::new([Cell::new(0), Cell::new(0)]);
    let (t, b) = (ticks.clone(), built.clone());
    let ui = list(vec![0usize, 1], move |&i| {
        b[i].set(b[i].get() + 1);
        let t = t.clone();
        cond(
            i == 0,
            rectangle().anim(move |_, _| t.set(t.get() + 1)),
            rectangle(),
        )
    });
    let mut cx = Context::new();
    update_layout(&mut cx, &ui);
    let root = cx.root_id();
    built[0].set(0);
    built[1].set(0);

    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(ticks.get(), 1);
    assert_eq!([built[0].get(), built[1].get()], [1, 0]);
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

/// A `state` view A, and beside it a view B, wrapped in another `state` view, whose width is
/// A's state: B reads a state that is neither enclosing it nor under it.
fn sibling_reader() -> impl View {
    let shared: Rc<Cell<Option<StateHandle<f32>>>> = Rc::new(Cell::new(None));
    let (a, b) = (shared.clone(), shared);
    hstack((
        state(
            || 10.0f32,
            move |w, _| {
                a.set(Some(w));
                rectangle().size([10.0, 10.0])
            },
        ),
        state(
            || (),
            move |_, _| {
                let b = b.clone();
                state(
                    || (),
                    move |_, cx| {
                        let w = cx[b.get().unwrap()];
                        rectangle().size([w, 10.0]).role(accesskit::Role::Button)
                    },
                )
            },
        ),
    ))
}

#[test]
fn test_layout_follows_a_state_read_from_elsewhere() {
    let ui = sibling_reader();
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    // Reuse every cache once, so the reads have to carry over from reused results.
    layout(&mut cx, &ui);

    let a = StateHandle::<f32>::new(cx.id_at(&[0, 0]));
    let reader = cx.id_at(&[0, 1, 0]);
    assert_eq!(cx.get_layout(reader).rect.width(), 10.0);
    cx[a] = 30.0;
    layout(&mut cx, &ui);
    assert_eq!(cx.get_layout(reader).rect.width(), 30.0);
}

#[test]
fn test_access_follows_a_state_read_from_elsewhere() {
    let ui = sibling_reader();
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();
    let mut nodes = vec![];
    ui.access(root, &mut cx, &mut nodes);
    cx.clear_dirty();

    let a = StateHandle::<f32>::new(cx.id_at(&[0, 0]));
    cx[a] = 30.0;
    nodes.clear();
    ui.access(root, &mut cx, &mut nodes);
    assert!(
        nodes
            .iter()
            .any(|(_, n)| n.role() == accesskit::Role::Button)
    );
}

#[test]
fn test_tapping_the_focused_view_keeps_the_caches() {
    let ui = focus(|_| rectangle());
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();
    let tap = Event::TouchBegin {
        id: 0,
        position: [50.0, 50.0].into(),
    };

    ui.process(&tap, root, &mut cx, &mut vec![]);
    assert_eq!(cx.focused_id, Some(root));
    let floor = cx.floor_gen;
    assert!(floor > 0);

    ui.process(&tap, root, &mut cx, &mut vec![]);
    assert_eq!(cx.floor_gen, floor);
}

/// A list of `n` rows that aren't `state` views, each holding a `state` with its height, and
/// counting how often each row is built.
fn plain_rows(n: usize, built: Rc<Vec<Cell<usize>>>) -> impl View {
    list_shared(Arc::new((0..n).collect()), move |&i| {
        built[i].set(built[i].get() + 1);
        vstack((state(|| 10.0f32, |h, cx| rectangle().size([10.0, cx[h]])),))
    })
}

fn row_height(cx: &Context, i: usize) -> StateHandle<f32> {
    StateHandle::new(cx.id_at(&[0, hh(&i), 0]))
}

fn builds(built: &[Cell<usize>]) -> Vec<usize> {
    built.iter().map(|b| b.replace(0)).collect()
}

#[test]
fn test_list_lays_out_only_written_items() {
    let built = Rc::new((0..4).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let ui = plain_rows(4, built.clone());
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    assert_eq!(builds(&built), [1, 1, 1, 1]);

    // A write that leaves the row's size: only that row is laid out again, and every row
    // keeps its place.
    let offsets = |cx: &mut Context| -> Vec<f32> {
        (0..4)
            .map(|i| {
                let id = cx.child(cx.root_id(), i);
                cx.get_layout(id).offset.y
            })
            .collect()
    };
    let before = offsets(&mut cx);
    let h = row_height(&cx, 2);
    cx[h] = 10.0;
    layout(&mut cx, &ui);
    assert_eq!(builds(&built), [0, 0, 1, 0]);
    assert_eq!(offsets(&mut cx), before);

    // Nothing written: no row is built.
    layout(&mut cx, &ui);
    assert_eq!(builds(&built), [0, 0, 0, 0]);
}

#[test]
fn test_list_item_that_changes_size_moves_the_others() {
    let built = Rc::new((0..3).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let ui = plain_rows(3, built.clone());
    let mut cx = Context::new();
    layout(&mut cx, &ui);

    let h = row_height(&cx, 0);
    cx[h] = 30.0;
    layout(&mut cx, &ui);
    // Row 0 is at the top (y is up): 30 tall above two rows of 10.
    let root = cx.root_id();
    let offset = |cx: &mut Context, i: usize| {
        let id = cx.child(root, i);
        cx.get_layout(id).offset.y
    };
    assert_eq!(offset(&mut cx, 0), 20.0);
    assert_eq!(offset(&mut cx, 1), 10.0);
    assert_eq!(offset(&mut cx, 2), 0.0);
}

#[test]
fn test_list_lays_out_every_item_after_a_write_around_it() {
    let built = Rc::new((0..3).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let b = built.clone();
    // The rows show the state of the `state` view around the list.
    let ui = state(
        || 10.0f32,
        move |height, _| {
            let b = b.clone();
            list_shared(Arc::new((0..3usize).collect()), move |&i| {
                b[i].set(b[i].get() + 1);
                vstack((state(
                    || (),
                    move |_, cx| rectangle().size([10.0, cx[height]]),
                ),))
            })
        },
    );
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    builds(&built);

    let height = StateHandle::<f32>::new(cx.root_id());
    cx[height] = 20.0;
    layout(&mut cx, &ui);
    assert_eq!(builds(&built), [1, 1, 1]);
    assert_eq!(cx.get_layout(cx.root_id()).rect.height(), 60.0);
}

#[test]
fn test_list_dirty_walk_visits_only_written_items() {
    let built = Rc::new((0..4).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let ui = plain_rows(4, built.clone());
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();
    ui.dirty(root, LocalToWorld::identity(), &mut cx);
    cx.clear_dirty();
    cx.dirty_region.clear();

    let h = row_height(&cx, 1);
    cx[h] = 10.0;
    layout(&mut cx, &ui);
    builds(&built);
    ui.dirty(root, LocalToWorld::identity(), &mut cx);
    assert_eq!(builds(&built), [0, 1, 0, 0]);
    assert_eq!(cx.dirty_region.rects().len(), 1);
}

#[test]
fn test_anim_pass_visits_only_written_items() {
    let ticks = Rc::new(Cell::new(0));
    let built = Rc::new((0..3).map(|_| Cell::new(0)).collect::<Vec<_>>());
    let (t, b) = (ticks.clone(), built.clone());
    // Rows that start an animation when their state turns on.
    let ui = list_shared(Arc::new((0..3usize).collect()), move |&i| {
        b[i].set(b[i].get() + 1);
        let t = t.clone();
        vstack((state(
            || false,
            move |on, cx| {
                let t = t.clone();
                cond(
                    cx[on],
                    rectangle().anim(move |_, _| t.set(t.get() + 1)),
                    rectangle(),
                )
            },
        ),))
    });
    let mut cx = Context::new();
    update_layout(&mut cx, &ui);
    let root = cx.root_id();
    builds(&built);

    // Row 1 turns on: the pass, before any layout, visits only row 1 and finds its new `anim`.
    let on = StateHandle::<bool>::new(cx.id_at(&[0, hh(&1usize), 0]));
    cx[on] = true;
    ui.process(&Event::Anim, root, &mut cx, &mut vec![]);
    assert_eq!(builds(&built), [0, 1, 0]);
    assert_eq!(ticks.get(), 1);
}

#[test]
fn test_long_list_estimates_the_rows_not_laid_out() {
    // 1,000 rows, alternately 10 and 30 tall, in a 200 tall space.
    let ui = list_shared(Arc::new((0..1000usize).collect()), |&i| {
        rectangle().size([10.0, if i % 2 == 0 { 10.0 } else { 30.0 }])
    });
    let mut cx = Context::new();
    layout(&mut cx, &ui);
    let root = cx.root_id();

    // Laid out down to 400, two spaces' worth; the other 980 rows count at the average, 20.
    assert_eq!(cx.list_measured(root, 1000), 20);
    assert_eq!(cx.list_extent(root).size.height, 400.0 + 20.0 * 980.0);
    // Row 0 is at the top (y is up), row 1 below it.
    let row = |cx: &mut Context, i| {
        let id = cx.list_child(root, i);
        cx.get_layout(id).offset.y
    };
    assert_eq!(row(&mut cx, 0), 20_000.0 - 10.0);
    assert_eq!(row(&mut cx, 1), 20_000.0 - 40.0);
    // Rows not laid out have no nodes: two for each of the 20, and the list's.
    assert_eq!(cx.live_nodes(), 41);
}

#[test]
fn test_commands_reach_list_items_before_any_layout() {
    // The menu can be gathered before anything is laid out: every item counts then.
    let ui = hlist(vec![0usize, 1], |&i| {
        rectangle().command(if i == 0 { "Edit:A" } else { "Edit:B" }, None, |_| {})
    });
    let mut cx = Context::new();
    let mut cmds = vec![];
    cx.commands(&ui, &mut cmds);
    assert_eq!(cmds.len(), 2);
}
