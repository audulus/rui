use crate::*;
use std::any::Any;
use std::hash::Hash;

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

impl<ID, V, F> DynView for List<ID, F>
where
    ID: Hash + Clone + 'static,
    V: View,
    F: Fn(&ID) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        for child in self.ids.iter().rev() {
            path.push(hh(child));
            let offset = cx.get_layout(path).offset;
            ((self.func)(child)).process(&event.offset(-offset), path, cx, actions);
            path.pop();
        }
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        for child in &self.ids {
            path.push(hh(child));
            let offset = args.cx.get_layout(path).offset;

            args.vger.save();

            args.vger.translate(offset);

            ((self.func)(child)).draw(path, args);

            args.vger.restore();
            path.pop();
        }
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        match self.orientation {
            ListOrientation::Horizontal => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut sizes = Vec::<LocalSize>::with_capacity(self.ids.len());

                let mut width_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size =
                        ((self.func)(child)).layout(path, &mut args.size(proposed_child_size));
                    sizes.push(child_size);
                    path.pop();

                    width_sum += child_size.width;
                }

                let mut max_height = 0.0;
                for size in &sizes {
                    max_height = size.height.max(max_height)
                }

                let mut x = 0.0;
                for (id, child_size) in self.ids.iter().zip(sizes.iter()) {
                    path.push(hh(id));

                    let child_offset = align_v(
                        LocalRect::new(LocalPoint::origin(), *child_size),
                        LocalRect::new([x, 0.0].into(), [child_size.width, max_height].into()),
                        VAlignment::Middle,
                    );

                    args.cx.set_layout_offset(path, child_offset);

                    path.pop();

                    x += child_size.width;
                }

                LocalSize::new(width_sum, max_height)
            }
            ListOrientation::Vertical => {
                let n = self.ids.len() as f32;
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);

                let mut sizes = Vec::<LocalSize>::with_capacity(self.ids.len());

                let mut height_sum = 0.0;
                for child in &self.ids {
                    path.push(hh(child));
                    let child_size =
                        ((self.func)(child)).layout(path, &mut args.size(proposed_child_size));
                    sizes.push(child_size);
                    path.pop();

                    height_sum += child_size.height;
                }

                let mut max_width = 0.0;
                for size in &sizes {
                    max_width = size.width.max(max_width)
                }

                let mut y = height_sum;
                for (id, child_size) in self.ids.iter().zip(sizes.iter()) {
                    path.push(hh(id));

                    let child_offset = align_h(
                        LocalRect::new(LocalPoint::origin(), *child_size),
                        LocalRect::new(
                            [0.0, y - child_size.height].into(),
                            [max_width, child_size.height].into(),
                        ),
                        HAlignment::Center,
                    );

                    args.cx.set_layout_offset(path, child_offset);
                    path.pop();

                    y -= child_size.height;
                }

                LocalSize::new(max_width, height_sum)
            }
            ListOrientation::Z => {
                for child in &self.ids {
                    path.push(hh(child));
                    ((self.func)(child)).layout(path, args);
                    path.pop();
                }
                args.sz
            }
        }
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        for child in &self.ids {
            path.push(hh(child));
            let offset = cx.get_layout(path).offset;
            let xf = xform.pre_translate(offset);
            ((self.func)(child)).dirty(path, xf, cx);
            path.pop();
        }
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let mut hit = None;
        for child in &self.ids {
            path.push(hh(child));
            let offset = cx.get_layout(path).offset;

            if let Some(h) = ((self.func)(child)).hittest(path, pt - offset, cx) {
                hit = Some(h)
            }
            path.pop();
        }
        hit
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        for child in &self.ids {
            path.push(hh(child));
            ((self.func)(child)).commands(path, cx, cmds);
            path.pop();
        }
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        for child in &self.ids {
            path.push(hh(child));
            map.push(cx.view_id(path));
            ((self.func)(child)).gc(path, cx, map);
            path.pop();
        }
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);

        let children: Vec<accesskit::NodeId> = self
            .ids
            .iter()
            .filter_map(|child| {
                path.push(hh(child));
                let node_id = ((self.func)(child)).access(path, cx, nodes);
                path.pop();
                node_id
            })
            .collect();

        builder.set_children(children);
        nodes.push((cx.view_id(path).access_id(), builder.build()));
        Some(cx.view_id(path).access_id())
    }
}

impl<ID, F> private::Sealed for List<ID, F> {}

/// Displays a list of items all of which are represented by the same View. See `examples/list.rs`.
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
        let mut path = vec![0];
        let result = ui.layout(
            &mut path,
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
