use crate::*;
use std::any::Any;
use std::hash::Hash;

/// How far outside its bounds a child of a list may draw and still be drawn when its bounds are
/// just outside the visible area, as for a shadow.
const DRAW_SLACK: f32 = 64.0;

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
        self.sync(vid, cx);
        for (i, child) in self.ids.iter().enumerate().rev() {
            let cid = cx.child(vid, i);
            let offset = cx.get_layout(cid).offset;
            ((self.func)(child)).process(&event.offset(-offset), cid, cx, actions);
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        self.sync(vid, args.cx);
        for (i, child) in self.ids.iter().enumerate() {
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
                    let child_size =
                        ((self.func)(child)).layout(cid, &mut args.size(proposed_child_size));
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
                    let child_size =
                        ((self.func)(child)).layout(cid, &mut args.size(proposed_child_size));
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
                    ((self.func)(child)).layout(cid, args);
                }
                args.sz
            }
        }
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.sync(vid, cx);
        for (i, child) in self.ids.iter().enumerate() {
            let cid = cx.child(vid, i);
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
                ((self.func)(child)).access(cid, cx, nodes)
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
}
