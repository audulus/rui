use crate::*;
use std::any::Any;
use std::hash::Hash;

pub enum ListOrientation {
    Horizontal,
    Vertical,
    Z,
}

pub struct List<ID, F> {
    orientation: ListOrientation,
    ids: Vec<ID>,
    func: F,
}

impl<ID, V, F> View for List<ID, F>
where
    ID: Hash + 'static,
    V: View,
    F: Fn(&ID) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;
            ((self.func)(child)).process(&event.offset(-offset), child_id, cx, actions);
        }
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = args.cx.layout.entry(child_id).or_default().offset;

            args.vger.save();

            args.vger.translate(offset);

            ((self.func)(child)).draw(child_id, args);

            args.vger.restore();
        }
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        match self.orientation {
            ListOrientation::Horizontal => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut sizes = Vec::<LocalSize>::new();
                sizes.reserve(self.ids.len());

                let mut width_sum = 0.0;
                for child in &self.ids {
                    let child_id = id.child(child);
                    let child_size =
                        ((self.func)(child)).layout(child_id, &mut args.size(proposed_child_size));
                    sizes.push(child_size);

                    width_sum += child_size.width;
                }

                let mut max_height = 0.0;
                for size in &sizes {
                    max_height = size.height.max(max_height)
                }

                let mut x = 0.0;
                for c in 0..self.ids.len() {
                    let child_id = id.child(&self.ids[c]);
                    let child_size = sizes[c];

                    let child_offset = align_v(
                        LocalRect::new(LocalPoint::origin(), child_size),
                        LocalRect::new([x, 0.0].into(), [child_size.width, max_height].into()),
                        VAlignment::Middle,
                    );

                    args.cx.layout.entry(child_id).or_default().offset = child_offset;

                    x += child_size.width;
                }

                LocalSize::new(width_sum, max_height)
            }
            ListOrientation::Vertical => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);

                let mut sizes = Vec::<LocalSize>::new();
                sizes.reserve(self.ids.len());

                let mut height_sum = 0.0;
                for child in &self.ids {
                    let child_id = id.child(child);
                    let child_size =
                        ((self.func)(child)).layout(child_id, &mut args.size(proposed_child_size));
                    sizes.push(child_size);

                    height_sum += child_size.height;
                }

                let mut max_width = 0.0;
                for size in &sizes {
                    max_width = size.width.max(max_width)
                }

                let mut y = height_sum;
                for c in 0..self.ids.len() {
                    let child_id = id.child(&self.ids[c]);
                    let child_size = sizes[c];

                    let child_offset = align_h(
                        LocalRect::new(LocalPoint::origin(), child_size),
                        LocalRect::new(
                            [0.0, y - child_size.height].into(),
                            [max_width, child_size.height].into(),
                        ),
                        HAlignment::Center,
                    );

                    args.cx.layout.entry(child_id).or_default().offset = child_offset;

                    y -= child_size.height;
                }

                LocalSize::new(max_width, height_sum)
            }
            ListOrientation::Z => {
                for child in &self.ids {
                    let child_id = id.child(child);
                    ((self.func)(child)).layout(child_id, args);
                }
                args.sz
            }
        }
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;
            let xf = xform.pre_translate(offset);
            ((self.func)(child)).dirty(child_id, xf, cx);
        }
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let mut hit = None;
        for child in &self.ids {
            let child_id = id.child(child);
            let offset = cx.layout.entry(child_id).or_default().offset;

            if let Some(h) = ((self.func)(child)).hittest(child_id, pt - offset, cx) {
                hit = Some(h)
            }
        }
        hit
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            let child_id = id.child(child);
            ((self.func)(child)).commands(child_id, cx, cmds)
        }
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        for child in &self.ids {
            ((self.func)(child)).gc(id.child(child), cx, map)
        }
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);

        let children: Vec<accesskit::NodeId> = self
            .ids
            .iter()
            .filter_map(|child| ((self.func)(child)).access(id.child(child), cx, nodes))
            .collect();

        builder.set_children(children);
        nodes.push((id.access_id(), builder.build(&mut cx.access_node_classes)));
        Some(id.access_id())
    }
}

impl<ID, F> private::Sealed for List<ID, F> {}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
///
/// `ids` is a Vec of items that implement Hash.
///
/// `f` is a function called to generate a View for each item.
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
pub fn list<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Vertical,
        ids,
        func: f,
    }
}

pub fn hlist<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Horizontal,
        ids,
        func: f,
    }
}

pub fn zlist<ID: Hash, V: View, F: Fn(&ID) -> V + 'static>(ids: Vec<ID>, f: F) -> List<ID, F> {
    List {
        orientation: ListOrientation::Z,
        ids,
        func: f,
    }
}
