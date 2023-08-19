use crate::views::stack_layout::*;
use crate::*;
use std::any::Any;

enum StackOrientation {
    /// Views are stacked horizontally (right to left).
    Horizontal,

    /// Views are stacked vertically (top to bottom).
    Vertical,

    /// Views are stacked back to front.
    Z,
}

struct Stack<VT, D> {
    children: VT,
    phantom_direction: std::marker::PhantomData<D>,
}

trait StackDirection {
    const ORIENTATION: StackOrientation;
}
struct HorizontalDirection {}
impl StackDirection for HorizontalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Horizontal;
}
struct VerticalDirection {}
impl StackDirection for VerticalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Vertical;
}
struct ZDirection {}
impl StackDirection for ZDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Z;
}

impl<VT: ViewTuple + 'static, D: StackDirection + 'static> View for Stack<VT, D> {
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut c = self.children.len() as i64 - 1;
        self.children.foreach_view_rev(&mut |child| {
            path.push(c as u64);
            let offset = cx.get_layout(path).offset;
            (*child).process(&event.offset(-offset), path, cx, actions);
            path.pop();
            c -= 1;
        })
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            let layout_box = args.cx.get_layout(path);

            args.vger.save();

            args.vger.translate(layout_box.offset);

            (*child).draw(path, args);
            c += 1;

            if DEBUG_LAYOUT {
                let paint = args.vger.color_paint(CONTROL_BACKGROUND);
                args.vger.stroke_rect(
                    layout_box.rect.min(),
                    layout_box.rect.max(),
                    0.0,
                    1.0,
                    paint,
                );
            }

            path.pop();

            args.vger.restore();
        })
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let n = self.children.len() as f32;

        match D::ORIENTATION {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(path, proposed_child_size, args, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.width)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                let mut flex_length = 0.0;
                let length = stack_layout(
                    args.sz.width,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    path,
                    [flex_length, args.sz.height].into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_height = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_height = size.unwrap().height.max(max_height)
                }

                for c in 0..(self.children.len() as u64) {
                    let ab = intervals[c as usize];

                    let child_offset = align_v(
                        LocalRect::new(LocalPoint::origin(), child_sizes[c as usize].unwrap()),
                        LocalRect::new([ab.0, 0.0].into(), [ab.1 - ab.0, max_height].into()),
                        VAlignment::Middle,
                    );

                    path.push(c);
                    args.cx.set_layout_offset(path, child_offset);
                    path.pop();
                }

                [length, max_height].into()
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(path, proposed_child_size, args, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.height)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                let mut flex_length = 0.0;
                let length = stack_layout(
                    args.sz.height,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    path,
                    [args.sz.width, flex_length].into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_width = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_width = size.unwrap().width.max(max_width)
                }

                for c in 0..(self.children.len() as u64) {
                    let ab = intervals[c as usize];

                    let h = ab.1 - ab.0;
                    let child_offset = align_h(
                        LocalRect::new(LocalPoint::origin(), child_sizes[c as usize].unwrap()),
                        LocalRect::new([0.0, length - ab.0 - h].into(), [max_width, h].into()),
                        HAlignment::Center,
                    );

                    path.push(c);
                    args.cx.set_layout_offset(path, child_offset);
                    path.pop();
                }

                [max_width, length].into()
            }
            StackOrientation::Z => {
                let mut c = 0;
                self.children.foreach_view(&mut |child| {
                    path.push(c);
                    child.layout(path, args);
                    path.pop();
                    c += 1;
                });
                args.sz
            }
        }
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            let offset = cx.get_layout(path).offset;
            let xf = xform.pre_translate(offset);
            child.dirty(path, xf, cx);
            path.pop();
            c += 1;
        })
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            let offset = cx.get_layout(path).offset;

            if let Some(h) = child.hittest(path, pt - offset, cx) {
                hit = Some(h)
            }

            path.pop();

            c += 1;
        });
        hit
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            child.commands(path, cx, cmds);
            path.pop();
            c += 1;
        });
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            map.push(cx.view_id(path));
            child.gc(path, cx, map);
            path.pop();
            c += 1;
        });
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut c = 0;
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);
        let mut children = vec![];
        self.children.foreach_view(&mut |child| {
            path.push(c);
            if let Some(id) = child.access(path, cx, nodes) {
                children.push(id)
            }
            path.pop();
            c += 1;
        });
        builder.set_children(children);
        let aid = cx.view_id(path).access_id();
        nodes.push((aid, builder.build(&mut cx.access_node_classes)));
        Some(aid)
    }
}

impl<VT: ViewTuple, D: StackDirection> Stack<VT, D> {
    pub fn new(children: VT) -> Self {
        Self {
            children,
            phantom_direction: std::marker::PhantomData::default(),
        }
    }

    pub fn layout_fixed_children(
        &self,
        path: &mut IdPath,
        proposed_child_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            if !child.is_flexible() {
                child_sizes[c as usize] =
                    Some(child.layout(path, &mut args.size(proposed_child_size)))
            }
            path.pop();
            c += 1;
        });
    }

    pub fn layout_flex_children(
        &self,
        path: &mut IdPath,
        flex_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            path.push(c);
            if child.is_flexible() {
                child_sizes[c as usize] = Some(child.layout(path, &mut args.size(flex_size)));
            }
            path.pop();
            c += 1;
        });
    }
}

impl<VT, D> private::Sealed for Stack<VT, D> {}

/// Horizontal stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::<VT, HorizontalDirection>::new(children)
}

/// Vertical stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::<VT, VerticalDirection>::new(children)
}

/// Stack of up to 128 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::<VT, ZDirection>::new(children)
}
