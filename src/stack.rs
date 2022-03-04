use crate::*;

pub trait ViewTuple {
    fn foreach_view<F: FnMut(&dyn View)>(&self, f: &mut F);
    fn len(&self) -> usize;
}

pub enum StackOrientation {
    Horizontal,
    Vertical,
    Z,
}

pub struct Stack<VT: ViewTuple> {
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

    fn needs_redraw(&self, id: ViewID, cx: &mut Context) -> bool {
        let mut c = 0;
        let mut r = false;
        self.children.foreach_view(&mut |child| {
            r = r || (*child).needs_redraw(id.child(&c), cx);
            c += 1;
        });
        r
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        let mut c = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
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
            let offset = cx
                .layout
                .entry(child_id)
                .or_insert(LayoutBox::default())
                .offset;

            vger.save();

            vger.translate(offset);

            (*child).draw(child_id, cx, vger);
            c += 1;

            vger.restore();
        })
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.children.len() as f32;

        match self.orientation {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(sz.width / n, sz.height);

                let mut c = 0;
                let mut width_sum = 0.0;
                self.children.foreach_view(&mut |child| {
                    let child_id = id.child(&c);
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    cx.layout.entry(child_id).or_default().offset =
                        [width_sum, (sz.height - child_size.height) / 2.0].into();

                    width_sum += child_size.width;
                    c += 1;
                });

                LocalSize::new(width_sum, sz.height)
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(sz.width, sz.height / n);

                let mut c = 0;
                let mut y = sz.height;
                let mut height_sum = 0.0;
                self.children.foreach_view(&mut |child| {
                    let child_id = id.child(&c);
                    let child_size = child.layout(child_id, proposed_child_size, cx, vger);

                    y -= child_size.height;
                    cx.layout.entry(child_id).or_default().offset =
                        [(sz.width - child_size.width) / 2.0, y].into();

                    height_sum += child_size.height;
                    c += 1;
                });

                LocalSize::new(sz.width, height_sum)
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
                .or_insert(LayoutBox::default())
                .offset;

            if let Some(h) = child.hittest(child_id, pt - offset, cx, vger) {
                hit = Some(h)
            }

            c += 1;
        });
        hit
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
pub fn hstack<VT: ViewTuple>(children: VT) -> Stack<VT> {
    Stack::new(StackOrientation::Horizontal, children)
}

/// Vertical stack of up to 8 Views in a tuple. Each item can be a different view type.
pub fn vstack<VT: ViewTuple>(children: VT) -> Stack<VT> {
    Stack::new(StackOrientation::Vertical, children)
}

/// Stack of up to 8 overlaid Views in a tuple. Each item can be a different view type.
pub fn zstack<VT: ViewTuple>(children: VT) -> Stack<VT> {
    Stack::new(StackOrientation::Z, children)
}
