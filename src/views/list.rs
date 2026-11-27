use crate::*;
use std::any::Any;
use std::hash::Hash;
use std::ops::Range;

/// The first of `0..n` for which `pred` is false, where `pred` is true for some first part of
/// the range and false after it.
fn partition_point(n: usize, mut pred: impl FnMut(usize) -> bool) -> usize {
    let (mut lo, mut hi) = (0, n);
    while lo < hi {
        let mid = lo + (hi - lo) / 2;
        if pred(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

/// How far outside its bounds a child of a list may draw and still be drawn when its bounds are
/// just outside the visible area, as for a shadow. Pointer motion this close to an item also
/// reaches it.
const DRAW_SLACK: f32 = 64.0;

/// The items that were under the pointer at a list's last motion, press or release, kept in the
/// list's node. Not a `state`, so keeping it marks nothing written.
struct PointerItems(Vec<ViewId>);

#[derive(Clone, Copy)]
pub enum ListOrientation {
    Horizontal,
    Vertical,
    Z,
}

#[derive(Clone)]
pub struct List<ID, F> {
    orientation: ListOrientation,
    ids: Vec<ID>,
    func: F,
}

impl<ID, F> List<ID, F>
where
    ID: Hash,
{
    /// Lays out an item, or reuses its last size without building its view when the item is a
    /// `state` view whose layout is still valid.
    fn layout_child<V: View>(
        &self,
        id: &ID,
        cid: ViewId,
        sz: LocalSize,
        args: &mut LayoutArgs,
    ) -> LocalSize
    where
        F: Fn(&ID) -> V,
    {
        if let Some(size) = args.cx.cached_layout(cid, sz) {
            return size;
        }
        ((self.func)(id)).layout(cid, &mut args.size(sz))
    }

    /// The items, in order, that may meet `area`, in the list's local space, once their bounds
    /// are inflated by `DRAW_SLACK`. A vertical or horizontal list lays its items out in order
    /// along its axis, so a binary search finds them; a z-list stacks them all, so all of them.
    fn items_near(&self, vid: ViewId, cx: &mut Context, area: LocalRect) -> Range<usize> {
        // The bounds of item `i` along the list's axis, inflated.
        let mut span = |i: usize| {
            let cid = cx.child(vid, i);
            let layout = cx.get_layout(cid);
            let rect = layout
                .rect
                .translate(layout.offset)
                .inflate(DRAW_SLACK, DRAW_SLACK);
            match self.orientation {
                ListOrientation::Vertical => (rect.min_y(), rect.max_y()),
                _ => (rect.min_x(), rect.max_x()),
            }
        };
        let n = self.ids.len();
        match self.orientation {
            ListOrientation::Z => 0..n,
            // Item 0 is at the top, and y is up: the items before `start` lie above the
            // area, those from `end` on below it.
            ListOrientation::Vertical => {
                let start = partition_point(n, |i| span(i).0 > area.max_y());
                let end = partition_point(n, |i| span(i).1 >= area.min_y());
                start..end.max(start)
            }
            ListOrientation::Horizontal => {
                let start = partition_point(n, |i| span(i).1 < area.min_x());
                let end = partition_point(n, |i| span(i).0 <= area.max_x());
                start..end.max(start)
            }
        }
    }

    /// Matches the list's child nodes to its items, so child `i` is the node of `ids[i]`.
    fn sync(&self, vid: ViewId, cx: &mut Context) {
        cx.sync_keyed_children(vid, self.ids.iter().map(hh));
    }
}

impl<ID, V, F> DynView for List<ID, F>
where
    ID: Hash + Clone + 'static,
    V: View,
    F: Fn(&ID) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        // A list with no `anim` view in it has nothing to do in the `Event::Anim` pass, which
        // runs every update: return before visiting every item.
        if let Event::Anim = event
            && cx.skip_unmarked_anim(vid)
        {
            return;
        }

        self.sync(vid, cx);

        let position = match event {
            Event::TouchBegin { position, .. }
            | Event::TouchMove { position, .. }
            | Event::TouchEnd { position, .. }
            | Event::Scroll { position, .. } => Some(*position),
            _ => None,
        };

        let Some(position) = position else {
            for (i, child) in self.ids.iter().enumerate().rev() {
                let cid = cx.child(vid, i);
                if let Event::Anim = event
                    && (cx.skip_anim(cid) || cx.skip_unmarked_anim(cid))
                {
                    continue;
                }
                let offset = cx.get_layout(cid).offset;
                ((self.func)(child)).process(&event.offset(-offset), cid, cx, actions);
            }
            return;
        };

        // Motion and wheel events go only to the items they can concern: those under the
        // pointer, those that were under it at the last motion, press or release (so a hover
        // sees the pointer leave, even when a `scroll` moves it far away), and those holding a
        // touch (so a drag or a press that leaves its item still sees it move and end). Presses
        // and releases are rare and go to every item.
        let filter = matches!(event, Event::TouchMove { .. } | Event::Scroll { .. });
        let previous = cx
            .node_mut(vid)
            .state
            .take()
            .and_then(|s| s.downcast::<PointerItems>().ok())
            .map(|p| p.0)
            .unwrap_or_default();
        let holders: Vec<ViewId> = if filter {
            cx.touches
                .iter()
                .filter(|t| !t.is_default())
                .filter_map(|t| cx.child_toward(vid, *t))
                .collect()
        } else {
            vec![]
        };

        // The items that motion or a wheel event may concern, last first as for every event:
        // those near the pointer, and those that were under it or hold a touch.
        let items: Vec<usize> = if filter {
            let point = LocalRect::new(position, LocalSize::zero());
            let mut items: Vec<usize> = self.items_near(vid, cx, point).collect();
            items.extend(
                previous
                    .iter()
                    .chain(&holders)
                    .filter_map(|id| cx.child_index(vid, *id)),
            );
            items.sort_unstable_by(|a, b| b.cmp(a));
            items.dedup();
            items
        } else {
            (0..self.ids.len()).rev().collect()
        };

        let mut under = vec![];
        for i in items {
            let child = &self.ids[i];
            let cid = cx.child(vid, i);
            let layout = cx.get_layout(cid);
            let offset = layout.offset;

            // A z-list keeps no child sizes, so its items count as under the pointer, as for
            // drawing.
            let inside = layout.rect.size == LocalSize::zero()
                || layout
                    .rect
                    .translate(offset)
                    .inflate(DRAW_SLACK, DRAW_SLACK)
                    .contains(position);
            let nearby = !inside && (previous.contains(&cid) || holders.contains(&cid));
            if filter && !inside && !nearby {
                continue;
            }

            let view = (self.func)(child);
            view.process(&event.offset(-offset), cid, cx, actions);

            // An item reached while the pointer is beyond the slack, but still over a part
            // of it drawn outside its bounds (as with `offset`), stays under the pointer, so
            // it sees the pointer leave that part too.
            if inside
                || (nearby && cx.is_live(cid) && view.hittest(cid, position - offset, cx).is_some())
            {
                under.push(cid);
            }
        }

        // An item can remove the list while processing (as by closing its window). The items
        // under the pointer are kept from the last motion, press or release, not from a wheel
        // event: scrolling moves the content under a still pointer, and the item hovered before
        // must still see the next motion.
        if cx.is_live(vid) {
            let items = if matches!(event, Event::Scroll { .. }) {
                previous
            } else {
                under
            };
            cx.node_mut(vid).state = Some(Box::new(PointerItems(items)));
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        self.sync(vid, args.cx);
        let items = match args.cx.local_visible(args.vger) {
            Some(area) => self.items_near(vid, args.cx, area),
            None => 0..self.ids.len(),
        };
        for i in items {
            let child = &self.ids[i];
            let cid = args.cx.child(vid, i);
            let layout = args.cx.get_layout(cid);
            let offset = layout.offset;

            // Skip a child that lies entirely outside the visible area, with some slack for
            // shadows and other drawing just outside its bounds. A z-list keeps no child sizes,
            // so it draws everything.
            if layout.rect.size != LocalSize::zero() {
                let bounds = layout
                    .rect
                    .translate(offset)
                    .inflate(DRAW_SLACK, DRAW_SLACK);
                if !args.cx.is_visible(bounds, args.vger) {
                    continue;
                }
            }

            args.vger.save();

            args.vger.translate(offset);

            ((self.func)(child)).draw(cid, args);

            args.vger.restore();
        }
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.sync(vid, args.cx);
        match self.orientation {
            ListOrientation::Horizontal => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut sizes = Vec::<LocalSize>::with_capacity(self.ids.len());

                let mut width_sum = 0.0;
                for (i, child) in self.ids.iter().enumerate() {
                    let cid = args.cx.child(vid, i);
                    let child_size = self.layout_child(child, cid, proposed_child_size, args);
                    sizes.push(child_size);

                    width_sum += child_size.width;
                }

                let mut max_height = 0.0;
                for size in &sizes {
                    max_height = size.height.max(max_height)
                }

                let mut x = 0.0;
                for (i, child_size) in sizes.iter().enumerate() {
                    let cid = args.cx.child(vid, i);

                    let child_offset = align_v(
                        LocalRect::new(LocalPoint::origin(), *child_size),
                        LocalRect::new([x, 0.0].into(), [child_size.width, max_height].into()),
                        VAlignment::Middle,
                    );

                    // Keep the child's size with its offset, so drawing can skip children
                    // outside the visible area.
                    args.cx.update_layout(
                        cid,
                        LayoutBox {
                            rect: LocalRect::new(LocalPoint::origin(), *child_size),
                            offset: child_offset,
                        },
                    );

                    x += child_size.width;
                }

                LocalSize::new(width_sum, max_height)
            }
            ListOrientation::Vertical => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);

                let mut sizes = Vec::<LocalSize>::with_capacity(self.ids.len());

                let mut height_sum = 0.0;
                for (i, child) in self.ids.iter().enumerate() {
                    let cid = args.cx.child(vid, i);
                    let child_size = self.layout_child(child, cid, proposed_child_size, args);
                    sizes.push(child_size);

                    height_sum += child_size.height;
                }

                let mut max_width = 0.0;
                for size in &sizes {
                    max_width = size.width.max(max_width)
                }

                let mut y = height_sum;
                for (i, child_size) in sizes.iter().enumerate() {
                    let cid = args.cx.child(vid, i);

                    let child_offset = align_h(
                        LocalRect::new(LocalPoint::origin(), *child_size),
                        LocalRect::new(
                            [0.0, y - child_size.height].into(),
                            [max_width, child_size.height].into(),
                        ),
                        HAlignment::Center,
                    );

                    // Keep the child's size with its offset, so drawing can skip children
                    // outside the visible area.
                    args.cx.update_layout(
                        cid,
                        LayoutBox {
                            rect: LocalRect::new(LocalPoint::origin(), *child_size),
                            offset: child_offset,
                        },
                    );

                    y -= child_size.height;
                }

                LocalSize::new(max_width, height_sum)
            }
            ListOrientation::Z => {
                for (i, child) in self.ids.iter().enumerate() {
                    let cid = args.cx.child(vid, i);
                    let sz = args.sz;
                    self.layout_child(child, cid, sz, args);
                }
                args.sz
            }
        }
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        if !cx.is_subtree_dirty(vid) {
            return;
        }
        self.sync(vid, cx);
        for (i, child) in self.ids.iter().enumerate() {
            let cid = cx.child(vid, i);
            if !cx.is_subtree_dirty(cid) {
                continue;
            }
            let offset = cx.get_layout(cid).offset;
            let xf = xform.pre_translate(offset);
            ((self.func)(child)).dirty(cid, xf, cx);
        }
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.sync(vid, cx);
        let mut hit = None;
        for (i, child) in self.ids.iter().enumerate() {
            let cid = cx.child(vid, i);
            let offset = cx.get_layout(cid).offset;

            if let Some(h) = ((self.func)(child)).hittest(cid, pt - offset, cx) {
                hit = Some(h)
            }
        }
        hit
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.sync(vid, cx);
        for (i, child) in self.ids.iter().enumerate() {
            let cid = cx.child(vid, i);
            ((self.func)(child)).commands(cid, cx, cmds);
        }
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.sync(vid, cx);
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);

        let children: Vec<accesskit::NodeId> = self
            .ids
            .iter()
            .enumerate()
            .filter_map(|(i, child)| {
                let cid = cx.child(vid, i);
                // An item whose nodes were sent before and haven't changed is skipped without
                // building its view.
                cx.cached_access(cid)
                    .unwrap_or_else(|| ((self.func)(child)).access(cid, cx, nodes))
            })
            .collect();

        builder.set_children(children);
        nodes.push((vid.access_id(), builder.build()));
        Some(vid.access_id())
    }
}

impl<ID, F> private::Sealed for List<ID, F> {}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
///
/// While rendering, items that lie entirely outside the visible area (the window, narrowed by
/// any `clip` or `scroll` around the list) are not drawn. They are still laid out and still
/// receive events.
///
/// Pointer motion and wheel events reach only the items near the pointer, the items that were
/// under it at the last motion, press or release, and the items holding a touch, as for a drag;
/// presses, releases and every other event reach all items. An item that takes motion more than
/// 64 points outside its bounds gets it there only when the pointer moves there from over it.
///
/// `ids` is a Vec of items that implement Hash.
///
/// `f` is a function called to generate a DynView for each item.
///
/// For example:
///
/// ```no_run
/// # use rui::*;
/// rui(list(vec![1, 2, 3], |i| {
///     hstack((
///         circle(),
///         text(&format!("{:?}", i))
///     ))
/// }));
/// ```
pub fn list<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Vertical,
        ids,
        func: f,
    }
}

pub fn hlist<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Horizontal,
        ids,
        func: f,
    }
}

pub fn zlist<ID: Hash + Clone, V: View, F: Fn(&ID) -> V + Clone + 'static>(
    ids: Vec<ID>,
    f: F,
) -> List<ID, F> {
    List {
        orientation: ListOrientation::Z,
        ids,
        func: f,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_list_layout() {
        let mut cx = Context::new();
        let ui = list(vec![0, 1, 2], |_id| rectangle().size([40.0, 20.0]));
        let sz = [200.0, 200.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // 3 items each 20px tall, max width 40
        assert_eq!(result.width, 40.0);
        assert_eq!(result.height, 60.0);
    }

    #[test]
    fn test_horizontal_list_layout() {
        let mut cx = Context::new();
        let ui = hlist(vec![0, 1, 2], |_id| rectangle().size([30.0, 50.0]));
        let sz = [200.0, 200.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // 3 items each 30px wide, max height 50
        assert_eq!(result.width, 90.0);
        assert_eq!(result.height, 50.0);
    }

    #[test]
    fn test_z_list_layout() {
        let mut cx = Context::new();
        let ui = zlist(vec![0, 1], |_id| rectangle());
        let sz = [100.0, 80.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Z layout returns the proposed size
        assert_eq!(result, sz);
    }

    #[test]
    fn test_list_empty() {
        let mut cx = Context::new();
        let ui = list(Vec::<i32>::new(), |_id| rectangle());
        let sz = [100.0, 100.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(result.width, 0.0);
        assert_eq!(result.height, 0.0);
    }

    #[test]
    fn test_vertical_list_varying_heights() {
        let mut cx = Context::new();
        let ui = list(vec![0, 1, 2], |id| {
            let h = (*id + 1) as f32 * 10.0;
            rectangle().size([50.0, h])
        });
        let sz = [200.0, 200.0].into();
        let root = cx.root_id();
        let result = ui.layout(
            root,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        // Heights: 10 + 20 + 30 = 60
        assert_eq!(result.height, 60.0);
        assert_eq!(result.width, 50.0);
    }

    // Pointer events and the items they reach. Each test gives its layout in rui's y-up
    // coordinates, where the first item of a list is at the top.

    use crate::test_harness::TestHarness;

    fn moved(position: [f32; 2]) -> Event {
        Event::TouchMove {
            id: 0,
            position: position.into(),
            delta: LocalOffset::zero(),
        }
    }

    #[test]
    fn test_drag_that_leaves_its_item_keeps_going() {
        // Items 200 tall, so each is far from the others even with the slack: item 0 is at
        // y=400..600, item 1 at 200..400, item 2 at 0..200.
        let ui = state(Vec::<(usize, GestureState)>::new, |log, _| {
            list(vec![0usize, 1, 2], move |&i| {
                rectangle()
                    .size([100.0, 200.0])
                    .drag(move |cx, _, gesture, _| cx[log].push((i, gesture)))
            })
        });
        let mut h = TestHarness::new(&ui, [100.0, 600.0]);
        let log = StateHandle::<Vec<(usize, GestureState)>>::new(h.cx.root_id());

        h.event(&Event::TouchBegin {
            id: 0,
            position: [50.0, 500.0].into(),
        });
        // Onto item 2, then out of the list altogether.
        h.event(&moved([50.0, 100.0]));
        h.event(&moved([50.0, -5000.0]));
        h.event(&Event::TouchEnd {
            id: 0,
            position: [50.0, -5000.0].into(),
        });

        assert_eq!(
            h.cx[log],
            vec![
                (0, GestureState::Began),
                (0, GestureState::Changed),
                (0, GestureState::Changed),
                (0, GestureState::Ended),
            ]
        );

        // The touch was released, so another item can take the next one.
        h.drag([50.0, 300.0], [50.0, 310.0]);
        assert_eq!(h.cx[log][4], (1, GestureState::Began));
    }

    #[test]
    fn test_tap_released_outside_its_item_frees_the_touch() {
        let ui = state(Vec::<usize>::new, |taps, _| {
            list(vec![0usize, 1, 2], move |&i| {
                rectangle()
                    .size([100.0, 30.0])
                    .tap(move |cx| cx[taps].push(i))
            })
        });
        // Items 30 tall: item 0 is at y=60..90, item 1 at 30..60, item 2 at 0..30.
        let mut h = TestHarness::new(&ui, [100.0, 90.0]);
        let taps = StateHandle::<Vec<usize>>::new(h.cx.root_id());

        h.event(&Event::TouchBegin {
            id: 0,
            position: [50.0, 75.0].into(),
        });
        h.event(&moved([50.0, -500.0]));
        h.event(&Event::TouchEnd {
            id: 0,
            position: [50.0, -500.0].into(),
        });
        assert!(h.cx.touches[0].is_default());

        h.tap([50.0, 45.0]);
        assert_eq!(h.cx[taps].last(), Some(&1));
    }

    /// Three rows that each keep whether they are hovered and count their hover calls, in a
    /// 100x90 list: item 0 is at y=60..90, item 1 at 30..60, item 2 at 0..30.
    fn hover_rows() -> impl View {
        state(
            || ([false; 3], [0u32; 3]),
            |rows, _| {
                list(vec![0usize, 1, 2], move |&i| {
                    rectangle().size([100.0, 30.0]).hover(move |cx, inside| {
                        cx[rows].0[i] = inside;
                        cx[rows].1[i] += 1;
                    })
                })
            },
        )
    }

    #[test]
    fn test_hover_sees_the_pointer_leave() {
        let ui = hover_rows();
        let mut h = TestHarness::new(&ui, [100.0, 90.0]);
        let rows = StateHandle::<([bool; 3], [u32; 3])>::new(h.cx.root_id());

        h.event(&moved([50.0, 75.0]));
        assert_eq!(h.cx[rows].0, [true, false, false]);

        // Far onto item 2: item 0, under the pointer before, still hears of it.
        h.event(&moved([50.0, 15.0]));
        assert_eq!(h.cx[rows].0, [false, false, true]);

        // Far away, as a `scroll` sends motion outside its viewport.
        h.event(&moved([f32::MIN, f32::MIN]));
        assert_eq!(h.cx[rows].0, [false, false, false]);
    }

    #[test]
    fn test_motion_skips_items_far_from_the_pointer() {
        let ui = state(
            || [0u32; 10],
            |calls, _| {
                list((0..10usize).collect(), move |&i| {
                    rectangle()
                        .size([100.0, 100.0])
                        .hover(move |cx, _| cx[calls][i] += 1)
                })
            },
        );
        let mut h = TestHarness::new(&ui, [100.0, 1000.0]);
        let calls = StateHandle::<[u32; 10]>::new(h.cx.root_id());

        // Item 0 is at the top, y=900..1000; item 1 ends at 900, plus the 64 points of slack.
        h.event(&moved([50.0, 980.0]));
        h.event(&moved([50.0, 990.0]));
        assert_eq!(h.cx[calls], [2, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_hover_sees_the_pointer_leave_after_scrolling() {
        let ui = hover_rows();
        let mut h = TestHarness::new(&ui, [100.0, 90.0]);
        let rows = StateHandle::<([bool; 3], [u32; 3])>::new(h.cx.root_id());

        h.event(&moved([50.0, 75.0]));
        assert_eq!(h.cx[rows].0, [true, false, false]);

        // Wheel events with the pointer still: a `scroll` around the list moves the pointer,
        // in the list's coordinates, far from item 0.
        for y in [-200.0, -400.0] {
            h.event(&Event::Scroll {
                position: [50.0, y].into(),
                delta: [0.0, -200.0].into(),
            });
        }

        // The next motion still reaches item 0.
        h.event(&moved([50.0, -600.0]));
        assert_eq!(h.cx[rows].0, [false, false, false]);
    }

    /// Lays out `ui` in a 2000 x 2000 window.
    fn lay_out(ui: &impl View, cx: &mut Context) {
        let root = cx.root_id();
        ui.layout(
            root,
            &mut LayoutArgs {
                sz: [2000.0, 2000.0].into(),
                cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
    }

    fn area(x: f32, y: f32, width: f32, height: f32) -> LocalRect {
        LocalRect::new([x, y].into(), [width, height].into())
    }

    #[test]
    fn test_items_near_an_area_of_a_vertical_list() {
        // Item i covers y=1980-20i..2000-20i.
        let ui = list((0..100usize).collect(), |_| rectangle().size([40.0, 20.0]));
        let mut cx = Context::new();
        lay_out(&ui, &mut cx);
        let root = cx.root_id();

        // Inflated by 64, items 41 (y=1096..1180 before inflating) to 53 reach y=1000..1100.
        assert_eq!(
            ui.items_near(root, &mut cx, area(0.0, 1000.0, 40.0, 100.0)),
            41..54
        );
        assert_eq!(
            ui.items_near(root, &mut cx, area(0.0, 1990.0, 0.0, 0.0)),
            0..4
        );
        assert!(
            ui.items_near(root, &mut cx, area(0.0, 3000.0, 40.0, 10.0))
                .is_empty()
        );
        assert!(
            ui.items_near(root, &mut cx, area(0.0, -500.0, 40.0, 10.0))
                .is_empty()
        );
    }

    #[test]
    fn test_items_near_an_area_of_a_horizontal_list() {
        // Item i covers x=10i..10i+10.
        let ui = hlist((0..100usize).collect(), |_| rectangle().size([10.0, 20.0]));
        let mut cx = Context::new();
        lay_out(&ui, &mut cx);
        let root = cx.root_id();

        assert_eq!(
            ui.items_near(root, &mut cx, area(500.0, 0.0, 10.0, 20.0)),
            43..58
        );
        assert!(
            ui.items_near(root, &mut cx, area(1100.0, 0.0, 10.0, 20.0))
                .is_empty()
        );
    }

    #[test]
    fn test_items_near_an_area_of_a_z_list_are_all() {
        let ui = zlist((0..5usize).collect(), |_| rectangle().size([10.0, 20.0]));
        let mut cx = Context::new();
        lay_out(&ui, &mut cx);
        let root = cx.root_id();
        assert_eq!(
            ui.items_near(root, &mut cx, area(1500.0, 1500.0, 1.0, 1.0)),
            0..5
        );
    }

    #[test]
    fn test_hover_beyond_the_slack_sees_the_pointer_leave() {
        // One item 30 tall at y=0..30 whose hover area is moved 200 up, to y=200..230, beyond
        // the 64 points of slack.
        let ui = state(
            || false,
            |hovered, _| {
                list(vec![0usize], move |_| {
                    rectangle()
                        .size([100.0, 30.0])
                        .offset([0.0, 200.0])
                        .hover(move |cx, inside| cx[hovered] = inside)
                })
            },
        );
        let mut h = TestHarness::new(&ui, [100.0, 30.0]);
        let hovered = StateHandle::<bool>::new(h.cx.root_id());

        // Over the item, then up onto its hover area.
        h.event(&moved([50.0, 15.0]));
        h.event(&moved([50.0, 215.0]));
        assert!(h.cx[hovered]);

        h.event(&moved([50.0, -1000.0]));
        assert!(!h.cx[hovered]);
    }
}
