use crate::*;

/// Allows rui to iterate over a tuple of `Views`.
pub trait ViewTuple {
    fn foreach_view<F: FnMut(&dyn View)>(&self, f: &mut F);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        false
    } // satisfy clippy
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

impl<VT: ViewTuple + 'static> View for Stack<VT> {
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Stack {{");
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            (*child).print(id.child(&c), cx);
            c += 1;
        });
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;

            let mut local_event = event.clone();
            local_event.position -= offset;

            (*child).process(&local_event, child_id, cx, vger);
            c += 1;
        })
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let layout_box = *cx.layout.entry(child_id).or_default();

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

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
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

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
        region: &mut Region<WorldSpace>,
    ) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;
            let xf = xform.pre_translate(offset);
            child.dirty(child_id, xf, cx, region);
            c += 1;
        })
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        let mut c = 0;
        let mut hit = None;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx.layout.entry(child_id).or_default().offset;

            if let Some(h) = child.hittest(child_id, pt - offset, cx, vger) {
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

macro_rules! impl_view_tuple {
    ( $n: tt; $( $t:ident),* ; $( $s:tt ),* ) => {

        impl< $( $t: View, )* > ViewTuple for ( $( $t, )* ) {
            fn foreach_view<FN: FnMut(&dyn View)>(&self, f: &mut FN) {
                $( f(&self.$s); )*
            }
            fn len(&self) -> usize {
                $n
            }
        }
    }
}

impl_view_tuple!(1; V0; 0);
impl_view_tuple!(2; V0, V1; 0, 1);
impl_view_tuple!(3; V0, V1, V2; 0, 1, 2);
impl_view_tuple!(4; V0, V1, V2, V3; 0, 1, 2, 3);
impl_view_tuple!(5; V0, V1, V2, V3, V4; 0, 1, 2, 3, 4);
impl_view_tuple!(6; V0, V1, V2, V3, V4, V5; 0, 1, 2, 3, 4, 5);
impl_view_tuple!(7; V0, V1, V2, V3, V4, V5, V6; 0, 1, 2, 3, 4, 5, 6);
impl_view_tuple!(8;
    V0, V1, V2, V3, V4, V5, V6, V7;
    0, 1, 2, 3, 4, 5, 6, 7
);
impl_view_tuple!(9;
    V0, V1, V2, V3, V4, V5, V6, V7, V8;
    0, 1, 2, 3, 4, 5, 6, 7, 8
);
impl_view_tuple!(10;
    V0, V1, V2, V3, V4, V5, V6, V7, V8, V9;
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9
);

/// Horizontal stack of up to 8 Views in a tuple. Each item can be a different view type.
pub fn hstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Horizontal, children)
}

/// Vertical stack of up to 8 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Vertical, children)
}

/// Stack of up to 8 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple + 'static>(children: VT) -> impl View {
    Stack::new(StackOrientation::Z, children)
}
