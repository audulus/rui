use crate::views::stack_layout::*;
use crate::*;
use std::any::Any;

#[derive(Clone)]
pub enum StackOrientation {
    /// Views are stacked horizontally (right to left).
    Horizontal,

    /// Views are stacked vertically (top to bottom).
    Vertical,

    /// Views are stacked back to front.
    Z,
}

#[derive(Clone)]
pub struct Stack<VT, D> {
    children: VT,
    phantom_direction: std::marker::PhantomData<D>,
}

pub trait StackDirection: Clone {
    const ORIENTATION: StackOrientation;
}

#[derive(Clone)]
pub struct HorizontalDirection {}

impl StackDirection for HorizontalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Horizontal;
}

#[derive(Clone)]
pub struct VerticalDirection {}

impl StackDirection for VerticalDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Vertical;
}

#[derive(Clone)]
pub struct ZDirection {}

impl StackDirection for ZDirection {
    const ORIENTATION: StackOrientation = StackOrientation::Z;
}

impl<VT: ViewTuple + 'static, D: StackDirection + 'static> DynView for Stack<VT, D> {
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut c = self.children.len() as i64 - 1;
        self.children.foreach_view_rev(&mut |child| {
            let cid = cx.child(vid, c as usize);
            let offset = cx.get_layout(cid).offset;
            (*child).process(&event.offset(-offset), cid, cx, actions);
            c -= 1;
        })
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let cid = args.cx.child(vid, c as usize);
            let layout_box = args.cx.get_layout(cid);

            args.vger.save();

            args.vger.translate(layout_box.offset);

            (*child).draw(cid, args);
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

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let n = self.children.len() as f32;

        match D::ORIENTATION {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(args.sz.width / n, args.sz.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(vid, proposed_child_size, args, &mut child_sizes);

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
                    vid,
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

                    let cid = args.cx.child(vid, c as usize);
                    args.cx.set_layout_offset(cid, child_offset);
                }

                [length, max_height].into()
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(args.sz.width, args.sz.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(vid, proposed_child_size, args, &mut child_sizes);

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
                    vid,
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

                    let cid = args.cx.child(vid, c as usize);
                    args.cx.set_layout_offset(cid, child_offset);
                }

                [max_width, length].into()
            }
            StackOrientation::Z => {
                let mut c = 0;
                self.children.foreach_view(&mut |child| {
                    let cid = args.cx.child(vid, c as usize);
                    child.layout(cid, args);
                    c += 1;
                });
                args.sz
            }
        }
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let cid = cx.child(vid, c as usize);
            let offset = cx.get_layout(cid).offset;
            let xf = xform.pre_translate(offset);
            child.dirty(cid, xf, cx);
            c += 1;
        })
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            let cid = cx.child(vid, c as usize);
            let offset = cx.get_layout(cid).offset;

            if let Some(h) = child.hittest(cid, pt - offset, cx) {
                hit = Some(h)
            }

            c += 1;
        });
        hit
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let cid = cx.child(vid, c as usize);
            child.commands(cid, cx, cmds);
            c += 1;
        });
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let mut c = 0;
        let mut builder = accesskit::NodeBuilder::new(accesskit::Role::List);
        let mut children = vec![];
        self.children.foreach_view(&mut |child| {
            let cid = cx.child(vid, c as usize);
            if let Some(id) = child.access(cid, cx, nodes) {
                children.push(id)
            }
            c += 1;
        });
        builder.set_children(children);
        let aid = vid.access_id();
        nodes.push((aid, builder.build()));
        Some(aid)
    }
}

impl<VT: ViewTuple, D: StackDirection> Stack<VT, D> {
    pub fn new(children: VT) -> Self {
        Self {
            children,
            phantom_direction: std::marker::PhantomData,
        }
    }

    pub fn layout_fixed_children(
        &self,
        vid: ViewId,
        proposed_child_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let cid = args.cx.child(vid, c as usize);
            if !child.is_flexible() {
                child_sizes[c as usize] =
                    Some(child.layout(cid, &mut args.size(proposed_child_size)))
            }
            c += 1;
        });
    }

    pub fn layout_flex_children(
        &self,
        vid: ViewId,
        flex_size: LocalSize,
        args: &mut LayoutArgs,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let cid = args.cx.child(vid, c as usize);
            if child.is_flexible() {
                child_sizes[c as usize] = Some(child.layout(cid, &mut args.size(flex_size)));
            }
            c += 1;
        });
    }
}

impl<VT, D> private::Sealed for Stack<VT, D> {}

/// Horizontal stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, HorizontalDirection> {
    Stack::<VT, HorizontalDirection>::new(children)
}

/// Vertical stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, VerticalDirection> {
    Stack::<VT, VerticalDirection>::new(children)
}

/// Stack of up to 128 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple + 'static>(children: VT) -> Stack<VT, ZDirection> {
    Stack::<VT, ZDirection>::new(children)
}
