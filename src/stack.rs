use crate::*;

/// Allows rui to iterate over a tuple of `Views`.
pub trait ViewTuple {
    fn foreach_view<F: FnMut(&dyn View)>(&self, f: &mut F);
    fn len(&self) -> usize;
}

pub enum StackOrientation {
    Horizontal,
    Vertical,
    Z,
}

struct Stack<VT> {
    orientation: StackOrientation,
    children: VT,
}

impl<VT: ViewTuple> View for Stack<VT> {
    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("Stack {{");
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            (*child).print(id.child(&c), cx);
            c += 1;
        });
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_default()
                .offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            (*child).process(&local_event, child_id, cx, vger);
            c += 1;
        })
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let layout_box = *cx
                .layout
                .entry(child_id)
                .or_default();

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

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.children.len() as f32;

        match self.orientation {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(sz.width / n, sz.height);

                let mut c = 0;
                let mut x = 0.0;
                self.children.foreach_view(&mut |child| {
                    let child_id = id.child(&c);
                    let child_rect = LocalRect::new([x, 0.0].into(), proposed_child_size);
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    cx.layout.entry(child_id).or_default().offset = align_h(
                        LocalRect::new(LocalPoint::origin(), child_size),
                        child_rect,
                        HAlignment::Center,
                    );

                    x += proposed_child_size.width;
                    c += 1;
                });

                sz
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(sz.width, sz.height / n);

                let mut c = 0;
                let mut y = sz.height;
                self.children.foreach_view(&mut |child| {
                    let child_id = id.child(&c);
                    let child_rect = LocalRect::new(
                        [0.0, y - proposed_child_size.height].into(),
                        proposed_child_size,
                    );
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    y -= proposed_child_size.height;
                    cx.layout.entry(child_id).or_default().offset = align_v(
                        LocalRect::new(LocalPoint::origin(), child_size),
                        child_rect,
                        VAlignment::Middle,
                    );

                    c += 1;
                });

                sz
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

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_default()
                .offset;

            if let Some(h) = child.hittest(child_id, pt - offset, cx, vger) {
                hit = Some(h)
            }

            c += 1;
        });
        hit
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            child.commands(id.child(&c), cx, cmds);
            c += 1;
        });
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut Vec<ViewID>) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            child.gc(id.child(&c), cx, map);
            c += 1;
        });
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let mut c = 0;
        let mut node = accesskit::Node::new(id.access_id(), accesskit::Role::List);
        self.children.foreach_view(&mut |child| {
            match child.access(id.child(&c), cx, nodes) {
                Some(id) => node.children.push(id),
                None => (),
            };
            c += 1;
        });
        nodes.push(node);
        Some(id.access_id())
    }
}

impl<VT: ViewTuple> Stack<VT> {
    pub fn new(orientation: StackOrientation, children: VT) -> Self {
        Self {
            orientation,
            children,
        }
    }
}

impl<VT> private::Sealed for Stack<VT> {}

impl<A: View> ViewTuple for (A,) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
    }
    fn len(&self) -> usize {
        1
    }
}

impl<A: View, B: View> ViewTuple for (A, B) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
    }
    fn len(&self) -> usize {
        2
    }
}

impl<A: View, B: View, C: View> ViewTuple for (A, B, C) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
    }
    fn len(&self) -> usize {
        3
    }
}

impl<A: View, B: View, C: View, D: View> ViewTuple for (A, B, C, D) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
    }
    fn len(&self) -> usize {
        4
    }
}

impl<A: View, B: View, C: View, D: View, E: View> ViewTuple for (A, B, C, D, E) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
    }
    fn len(&self) -> usize {
        5
    }
}

impl<A: View, B: View, C: View, D: View, E: View, F: View> ViewTuple for (A, B, C, D, E, F) {
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
    }
    fn len(&self) -> usize {
        6
    }
}

impl<A: View, B: View, C: View, D: View, E: View, F: View, G: View> ViewTuple
    for (A, B, C, D, E, F, G)
{
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
        f(&self.6);
    }
    fn len(&self) -> usize {
        7
    }
}

impl<A: View, B: View, C: View, D: View, E: View, F: View, G: View, H: View> ViewTuple
    for (A, B, C, D, E, F, G, H)
{
    fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
        f(&self.0);
        f(&self.1);
        f(&self.2);
        f(&self.3);
        f(&self.4);
        f(&self.5);
        f(&self.6);
        f(&self.7);
    }
    fn len(&self) -> usize {
        8
    }
}

/// Horizontal stack of up to 8 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple>(children: VT) -> impl View {
    Stack::new(StackOrientation::Horizontal, children)
}

/// Vertical stack of up to 8 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple>(children: VT) -> impl View {
    Stack::new(StackOrientation::Vertical, children)
}

/// Stack of up to 8 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple>(children: VT) -> impl View {
    Stack::new(StackOrientation::Z, children)
}
