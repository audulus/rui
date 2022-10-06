use crate::*;
use std::any::Any;
use crate::views::stack_layout::*;

pub enum StackOrientation {
    Horizontal,
    Vertical,
    Z,
}

struct Stack<VT> {
    orientation: StackOrientation,
    children: VT,
}

/// Common functions shared between stack types.
trait StackCommon<VT: ViewTuple> {
    fn with_children(&self, f: impl FnOnce(&VT));

    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let mut c = 0;
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                let child_id = id.child(&c);
                let offset = cx.layout.entry(child_id).or_default().offset;
                (*child).process(&event.offset(-offset), child_id, cx, vger, actions);
                c += 1;
            })
        })
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let mut c = 0;
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                let child_id = id.child(&c);
                let offset = cx.layout.entry(child_id).or_default().offset;
                let xf = xform.pre_translate(offset);
                child.dirty(child_id, xf, cx);
                c += 1;
            })
        })
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                let child_id = id.child(&c);
                let offset = cx.layout.entry(child_id).or_default().offset;

                if let Some(h) = child.hittest(child_id, pt - offset, cx, vger) {
                    hit = Some(h)
                }

                c += 1;
            })
        });
        hit
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                child.commands(id.child(&c), cx, cmds);
                c += 1;
            })
        })
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        let mut c = 0;
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                child.gc(id.child(&c), cx, map);
                c += 1;
            })
        })
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let mut c = 0;
        let mut node = accesskit::Node::new(id.access_id(), accesskit::Role::List);
        self.with_children(&mut |children: &VT| {
            children.foreach_view(&mut |child| {
                if let Some(id) = child.access(id.child(&c), cx, nodes) {
                    node.children.push(id)
                }
                c += 1;
            })
        });
        nodes.push(node);
        Some(id.access_id())
    }
}

impl<VT: ViewTuple + 'static> StackCommon<VT> for Stack<VT> {
    fn with_children(&self, f: impl FnOnce(&VT)) {
        f(&self.children)
    }
}

impl<VT: ViewTuple + 'static> View for Stack<VT> {

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let layout_box = cx.layout[&child_id];

            vger.save();

            vger.translate(layout_box.offset);

            (*child).draw(child_id, cx, vger);
            c += 1;

            if DEBUG_LAYOUT {
                let paint = vger.color_paint(CONTROL_BACKGROUND);
                vger.stroke_rect(
                    layout_box.rect.min(),
                    layout_box.rect.max(),
                    0.0,
                    1.0,
                    paint,
                );
            }

            vger.restore();
        })
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        let n = self.children.len() as f32;

        match self.orientation {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(sz.width / n, sz.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(id, proposed_child_size, cx, vger, &mut child_sizes);

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
                    sz.width,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    id,
                    [flex_length, sz.height].into(),
                    cx,
                    vger,
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

                    cx.layout.entry(child_id).or_default().offset = child_offset;
                }

                [length, max_height].into()
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(sz.width, sz.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_fixed_children(id, proposed_child_size, cx, vger, &mut child_sizes);

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
                    sz.height,
                    &child_sizes_1d[0..n],
                    &mut intervals[0..n],
                    &mut flex_length,
                );

                self.layout_flex_children(
                    id,
                    [sz.width, flex_length].into(),
                    cx,
                    vger,
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

                    cx.layout.entry(child_id).or_default().offset = child_offset;
                }

                [max_width, length].into()
            }
            StackOrientation::Z => {
                let mut c = 0;
                self.children.foreach_view(&mut |child| {
                    child.layout(id.child(&c), sz, cx, vger);
                    c += 1;
                });
                sz
            }
        }
    }

}

impl<VT: ViewTuple> Stack<VT> {
    pub fn new(orientation: StackOrientation, children: VT) -> Self {
        Self {
            orientation,
            children,
        }
    }

    pub fn layout_fixed_children(
        &self,
        id: ViewId,
        proposed_child_size: LocalSize,
        cx: &mut Context,
        vger: &mut Vger,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c: i32 = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            if !child.is_flexible() {
                child_sizes[c as usize] =
                    Some(child.layout(child_id, proposed_child_size, cx, vger))
            }
            c += 1;
        });
    }

    pub fn layout_flex_children(
        &self,
        id: ViewId,
        flex_size: LocalSize,
        cx: &mut Context,
        vger: &mut Vger,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c: i32 = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            if child.is_flexible() {
                child_sizes[c as usize] = Some(child.layout(child_id, flex_size, cx, vger));
            }
            c += 1;
        });
    }
}

impl<VT> private::Sealed for Stack<VT> {}

/// Horizontal stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Horizontal, children)
}

/// Vertical stack of up to 128 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Vertical, children)
}

/// Stack of up to 128 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Z, children)
}
