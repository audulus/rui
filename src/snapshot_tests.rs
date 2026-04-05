use crate::*;

/// Helper to format a layout entry as a readable string.
fn fmt_layout(path: &IdPath, lb: &context::LayoutBox) -> String {
    format!(
        "path={:?} rect=[{:.1}, {:.1}, {:.1}, {:.1}] offset=[{:.1}, {:.1}]",
        path,
        lb.rect.origin.x,
        lb.rect.origin.y,
        lb.rect.size.width,
        lb.rect.size.height,
        lb.offset.x,
        lb.offset.y,
    )
}

/// Run layout on a view and return a snapshot-friendly string of all layout entries.
fn layout_snapshot(view: &impl View, sz: impl Into<LocalSize>) -> String {
    let mut cx = Context::new();
    let sz = sz.into();
    let mut path = vec![0];
    let root_size = view.layout(
        &mut path,
        &mut LayoutArgs {
            sz,
            cx: &mut cx,
            text_bounds: &mut |_, _, _| LocalRect::zero(),
        },
    );

    let entries = cx.layout_entries();
    let mut lines = vec![format!(
        "root_size=[{:.1}, {:.1}]",
        root_size.width, root_size.height
    )];
    for (path, lb) in &entries {
        lines.push(fmt_layout(path, lb));
    }
    lines.join("\n")
}

// --- Single views ---

#[test]
fn test_snapshot_rectangle() {
    let ui = rectangle();
    insta::assert_snapshot!(layout_snapshot(&ui, [100.0, 50.0]));
}

#[test]
fn test_snapshot_circle() {
    let ui = circle();
    insta::assert_snapshot!(layout_snapshot(&ui, [80.0, 60.0]));
}

#[test]
fn test_snapshot_rectangle_with_size() {
    let ui = rectangle().size([40.0, 30.0]);
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

// --- Padding ---

#[test]
fn test_snapshot_padded_rectangle() {
    let ui = rectangle().padding(10.0);
    insta::assert_snapshot!(layout_snapshot(&ui, [100.0, 100.0]));
}

// --- Offset ---

#[test]
fn test_snapshot_offset_rectangle() {
    let ui = rectangle().offset([15.0, 25.0]);
    insta::assert_snapshot!(layout_snapshot(&ui, [100.0, 100.0]));
}

// --- Cond ---

#[test]
fn test_snapshot_cond_true() {
    let ui = cond(
        true,
        rectangle().size([50.0, 50.0]),
        rectangle().size([100.0, 100.0]),
    );
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_cond_false() {
    let ui = cond(
        false,
        rectangle().size([50.0, 50.0]),
        rectangle().size([100.0, 100.0]),
    );
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

// --- Vertical list ---

#[test]
fn test_snapshot_vlist_uniform() {
    let ui = list(vec![1, 2, 3], |_| rectangle().size([60.0, 20.0]));
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_vlist_varying() {
    let ui = list(vec![1, 2, 3], |id| {
        let h = (*id as f32) * 15.0;
        rectangle().size([50.0, h])
    });
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

// --- Horizontal list ---

#[test]
fn test_snapshot_hlist() {
    let ui = hlist(vec![1, 2, 3], |_| rectangle().size([30.0, 40.0]));
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_hlist_varying() {
    let ui = hlist(vec![1, 2, 3], |id| {
        let w = (*id as f32) * 20.0;
        rectangle().size([w, 40.0])
    });
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

// --- Z list ---

#[test]
fn test_snapshot_zlist() {
    let ui = zlist(vec![1, 2], |_| rectangle());
    insta::assert_snapshot!(layout_snapshot(&ui, [100.0, 80.0]));
}

// --- Nested compositions ---

#[test]
fn test_snapshot_padded_sized_rectangle() {
    let ui = rectangle().size([60.0, 40.0]).padding(5.0);
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_vlist_of_padded() {
    let ui = list(vec![1, 2], |_| rectangle().size([80.0, 30.0]).padding(5.0));
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_nested_lists() {
    let ui = list(vec![1, 2], |_| {
        hlist(vec![10, 20], |_| rectangle().size([40.0, 25.0]))
    });
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

// --- Button ---

#[test]
fn test_snapshot_button() {
    let ui = button(text("click"), |_| {});
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 50.0]));
}

// --- Empty view and spacer ---

#[test]
fn test_snapshot_empty_view() {
    let ui = EmptyView {};
    insta::assert_snapshot!(layout_snapshot(&ui, [100.0, 100.0]));
}

// --- Multiple modifiers ---

#[test]
fn test_snapshot_size_then_offset() {
    let ui = rectangle().size([50.0, 50.0]).offset([10.0, 20.0]);
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}

#[test]
fn test_snapshot_size_then_padding() {
    let ui = rectangle().size([50.0, 50.0]).padding(15.0);
    insta::assert_snapshot!(layout_snapshot(&ui, [200.0, 200.0]));
}
