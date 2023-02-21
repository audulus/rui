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
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;
            (*child).process(&event.offset(-offset), child_id, cx, actions);
            c += 1;
        })
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let layout_box = *args.cx.layout.entry(child_id).or_default();

            args.vger.save();

            args.vger.translate(layout_box.offset);

            (*child).draw(child_id, args);
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

            args.vger.restore();
        })
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let n = self.children.len() as f32;

        match D::ORIENTATION {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(id, proposed_child_size, args, &mut child_sizes);

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
                    id,
                    [flex_length, args.sz.height].into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_height = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_height = size.unwrap().height.max(max_height)
                }

                for c in 0..(self.children.len() as i32) {
                    let child_id = id.child(&c);
                    let ab = intervals[c as usize];

                    let child_offset = align_v(
                        LocalRect::new(LocalPoint::origin(), child_sizes[c as usize].unwrap()),
                        LocalRect::new([ab.0, 0.0].into(), [ab.1 - ab.0, max_height].into()),
                        VAlignment::Middle,
                    );

                    args.cx.layout.entry(child_id).or_default().offset = child_offset;
                }

                [length, max_height].into()
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(id, proposed_child_size, args, &mut child_sizes);

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
                    id,
                    [args.sz.width, flex_length].into(),
                    args,
                    &mut child_sizes,
                );

                let mut max_width = 0.0;
                for size in &child_sizes[0..self.children.len()] {
                    max_width = size.unwrap().width.max(max_width)
                }

                for c in 0..(self.children.len() as i32) {
                    let child_id = id.child(&c);
                    let ab = intervals[c as usize];

                    let h = ab.1 - ab.0;
                    let child_offset = align_h(
                        LocalRect::new(LocalPoint::origin(), child_sizes[c as usize].unwrap()),
                        LocalRect::new([0.0, length - ab.0 - h].into(), [max_width, h].into()),
                        HAlignment::Center,
                    );

                    args.cx.layout.entry(child_id).or_default().offset = child_offset;
                }

                [max_width, length].into()
            }
            StackOrientation::Z => {
                let mut c = 0;
                self.children.foreach_view(&mut |child| {
                    child.layout(id.child(&c), args);
                    c += 1;
                });
                args.sz
            }
        }
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;
            let xf = xform.pre_translate(offset);
            child.dirty(child_id, xf, cx);
            c += 1;
        })
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;

            if let Some(h) = child.hittest(child_id, pt - offset, cx) {
                hit = Some(h)
            }

            c += 1;
        });
        hit
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            child.commands(id.child(&c), cx, cmds);
            c += 1;
        });
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            child.gc(id.child(&c), cx, map);
            c += 1;
        });
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut c = 0;
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);
        let mut children = vec![];
        self.children.foreach_view(&mut |child| {
            if let Some(id) = child.access(id.child(&c), cx, nodes) {
                children.push(id)
            }
            c += 1;
        });
        builder.set_children(children);
        nodes.push((id.access_id(), builder.build(&mut cx.access_node_classes)));
        Some(id.access_id())
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
        id: ViewId,
        proposed_child_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c: i32 = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            if !child.is_flexible() {
                child_sizes[c as usize] =
                    Some(child.layout(child_id, &mut args.size(proposed_child_size)))
            }
            c += 1;
        });
    }

    pub fn layout_flex_children(
        &self,
        id: ViewId,
        flex_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c: i32 = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            if child.is_flexible() {
                child_sizes[c as usize] = Some(child.layout(child_id, &mut args.size(flex_size)));
            }
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
