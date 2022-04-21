use crate::*;

pub enum StackOrientation {
    Horizontal,
    Vertical,
    Z,
}

pub enum StackItem {
    Fixed(f32),
    Flexible,
}

/// 1-D stack layout to make the algorithm clear.
pub fn stack_layout(total: f32, sizes: &[StackItem], intervals: &mut [(f32, f32)]) {

    assert_eq!(sizes.len(), intervals.len());

    // Count the number of spacers and total of fixed sizes.
    let mut spacers = 0;
    let mut sizes_sum = 0.0;
    for sz in sizes {
        match sz {
            StackItem::Flexible => spacers += 1,
            StackItem::Fixed(s) => sizes_sum += s,
        }
    }

    // length of spacer is remaining size divided equally
    let spacer_length = (total - sizes_sum) / (spacers as f32);

    let mut x = 0.0;
    for i in 0..sizes.len() {
        let sz = match sizes[i] {
            StackItem::Flexible => spacer_length,
            StackItem::Fixed(s) => {
                if spacers != 0 {
                    s
                } else {
                    total / (sizes.len() as f32)
                }
            }
        };

        intervals[i] = (x, x + sz);
        x += sz;
    }
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

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let n = self.children.len() as f32;

        match self.orientation {
            StackOrientation::Horizontal => {
                let proposed_child_size = LocalSize::new(sz.width / n, sz.height);

                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_children(id, proposed_child_size, cx, vger, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.width)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                stack_layout(sz.width, &child_sizes_1d[0..n], &mut intervals[0..n]);

                for c in 0..(self.children.len() as i32) {
                    let child_id = id.child(&c);
                    let ab = intervals[c as usize];

                    let child_offset = align_h(
                        LocalRect::new(
                            LocalPoint::origin(),
                            if let Some(sz) = child_sizes[c as usize] {
                                sz
                            } else {
                                LocalSize::zero()
                            },
                        ),
                        LocalRect::new([ab.0, 0.0].into(), [ab.1 - ab.0, sz.height].into()),
                        HAlignment::Center,
                    );

                    cx.layout.entry(child_id).or_default().offset = child_offset;
                }

                sz
            }
            StackOrientation::Vertical => {
                let proposed_child_size = LocalSize::new(sz.width, sz.height / n);
                let mut child_sizes = [None; VIEW_TUPLE_MAX_ELEMENTS];
                self.layout_children(id, proposed_child_size, cx, vger, &mut child_sizes);

                let child_sizes_1d = child_sizes.map(|x| {
                    if let Some(s) = x {
                        StackItem::Fixed(s.height)
                    } else {
                        StackItem::Flexible
                    }
                });
                let mut intervals = [(0.0, 0.0); VIEW_TUPLE_MAX_ELEMENTS];
                let n = self.children.len();
                stack_layout(sz.height, &child_sizes_1d[0..n], &mut intervals[0..n]);

                for c in 0..(self.children.len() as i32) {
                    let child_id = id.child(&c);
                    let ab = intervals[c as usize];

                    let h = ab.1 - ab.0;
                    let child_offset = align_h(
                        LocalRect::new(
                            LocalPoint::origin(),
                            if let Some(sz) = child_sizes[c as usize] {
                                sz
                            } else {
                                LocalSize::zero()
                            },
                        ),
                        LocalRect::new([0.0, sz.height - ab.0 - h].into(), [sz.width, h].into()),
                        HAlignment::Center,
                    );

                    cx.layout.entry(child_id).or_default().offset = child_offset;
                }

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

    pub fn layout_children(
        &self,
        id: ViewId,
        proposed_child_size: LocalSize,
        cx: &mut Context,
        vger: &mut VGER,
        child_sizes: &mut [Option<LocalSize>],
    ) {
        let mut c: i32 = 0;
        self.children.foreach_view(&mut |child| {
            let child_id = id.child(&c);
            child_sizes[c as usize] = if child.is_flexible() {
                None
            } else {
                Some(child.layout(child_id, proposed_child_size, cx, vger))
            };
            c += 1;
        });
    }
}

impl<VT> private::Sealed for Stack<VT> {}

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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_layout_basic() {
        use StackItem::Fixed;
        use StackItem::Flexible;
        {
            let sizes = [Fixed(1.0), Fixed(1.0)];
            let mut intervals = [(0.0, 0.0), (0.0, 0.0)];

            stack_layout(4.0, &sizes, &mut intervals);

            println!("intervals: {:?}", intervals);
        }

        {
            let sizes = [Fixed(1.0), Flexible, Fixed(1.0)];
            let mut intervals = [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];

            stack_layout(4.0, &sizes, &mut intervals);

            println!("intervals: {:?}", intervals);
        }

        {
            let sizes = [Fixed(1.0), Fixed(1.0), Flexible];
            let mut intervals = [(0.0, 0.0), (0.0, 0.0), (0.0, 0.0)];

            stack_layout(4.0, &sizes, &mut intervals);

            println!("intervals: {:?}", intervals);
        }
    }
}
