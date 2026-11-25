use crate::*;
use std::any::Any;

/// Struct for the `scroll` view.
#[derive(Clone)]
pub struct ScrollView<V> {
    child: V,
}

impl<V: View> ScrollView<V> {
    /// The scroll position: how far the top of the content is above the top of the view.
    fn position(&self, path: &IdPath, cx: &mut Context) -> StateHandle<f32> {
        let id = cx.view_id(path);
        cx.init_state(id, &|| 0.0f32);
        StateHandle::new(id)
    }

    /// The viewport and the offset of the content inside it.
    fn geometry(path: &mut IdPath, cx: &mut Context) -> (LocalRect, LocalOffset) {
        let viewport = cx.get_layout(path).rect;
        path.push(0);
        let offset = cx.get_layout(path).offset;
        path.pop();
        (viewport, offset)
    }
}

impl<V: View> DynView for ScrollView<V> {
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let (viewport, offset) = Self::geometry(path, cx);

        let inside = match event {
            Event::TouchBegin { position, .. }
            | Event::TouchMove { position, .. }
            | Event::TouchEnd { position, .. }
            | Event::Scroll { position, .. } => Some(viewport.contains(*position)),
            _ => None,
        };

        // Content that is scrolled out of view takes no touches: a press outside the viewport
        // does not reach it, and motion outside it reaches it far away, so hovers end.
        let child_event = match (event, inside) {
            (Event::TouchBegin { .. }, Some(false)) => None,
            (Event::TouchMove { id, delta, .. }, Some(false)) => Some(Event::TouchMove {
                id: *id,
                position: [f32::MIN, f32::MIN].into(),
                delta: *delta,
            }),
            _ => Some(event.offset(-offset)),
        };

        if let Some(child_event) = child_event {
            path.push(0);
            self.child.process(&child_event, path, cx, actions);
            path.pop();
        }

        if let (Event::Scroll { delta, .. }, Some(true)) = (event, inside) {
            path.push(0);
            let content = cx.get_layout(path).rect.size;
            path.pop();
            let max = (content.height - viewport.height()).max(0.0);
            let handle = self.position(path, cx);
            let new_position = (cx[handle] + delta.y).clamp(0.0, max);
            if new_position != cx[handle] {
                cx[handle] = new_position;
            }
        }
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        let (viewport, offset) = Self::geometry(path, args.cx);
        args.vger.save();
        args.vger.scissor(viewport);
        args.vger.translate(offset);
        path.push(0);
        self.child.draw(path, args);
        path.pop();
        args.vger.restore();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        let handle = self.position(path, args.cx);

        path.push(0);
        let content = self.child.layout(path, args);
        path.pop();

        // Keep the position in range when the content or the view changes size.
        let max = (content.height - args.sz.height).max(0.0);
        let position = args.cx[handle].min(max);

        // The content's top sits at the view's top, moved up by the position (y is up).
        path.push(0);
        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), content),
                offset: [0.0, args.sz.height - content.height + position].into(),
            },
        );
        path.pop();

        args.cx.update_layout(
            path,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        let (_, offset) = Self::geometry(path, cx);
        path.push(0);
        self.child.dirty(path, xform.pre_translate(offset), cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let (viewport, offset) = Self::geometry(path, cx);
        if !viewport.contains(pt) {
            return None;
        }
        path.push(0);
        let id = self.child.hittest(path, pt - offset, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(cx.view_id(path));
        path.push(0);
        map.push(cx.view_id(path));
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V> private::Sealed for ScrollView<V> {}

/// A vertical scroll view. It fills the space it is offered, lays its content out at that size,
/// keeps the content's top at its own top, and moves the content with the mouse wheel or
/// touchpad over it, within the content's height. Content outside the view is clipped and takes
/// no touches.
pub fn scroll(child: impl View) -> ScrollView<impl View> {
    ScrollView { child }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_harness::TestHarness;

    /// A scroll view of 100 px over a column of ten 50 px rows, the top one tappable.
    fn column() -> impl View {
        state(
            || 0,
            |taps, _| {
                scroll(vstack((
                    rectangle().size([100.0, 50.0]).tap(move |cx| cx[taps] += 1),
                    rectangle().size([100.0, 450.0]),
                )))
            },
        )
    }

    fn position(h: &mut TestHarness<impl View>) -> f32 {
        let id = h.cx.view_id(&IdPath::from(vec![0, 0]));
        h.cx[StateHandle::<f32>::new(id)]
    }

    fn taps(h: &mut TestHarness<impl View>) -> i32 {
        let id = h.cx.view_id(&IdPath::root());
        h.cx[StateHandle::<i32>::new(id)]
    }

    #[test]
    fn test_scroll_top_aligns_and_clamps() {
        let ui = column();
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // The top row covers the top 50 px of the view (y is up).
        h.tap([50.0, 75.0]);
        assert_eq!(taps(&mut h), 1);

        let scroll = |h: &mut TestHarness<_>, dy: f32| {
            h.event(&Event::Scroll {
                position: [50.0, 50.0].into(),
                delta: [0.0, dy].into(),
            });
            h.layout();
        };
        scroll(&mut h, 30.0);
        assert_eq!(position(&mut h), 30.0);
        scroll(&mut h, 1000.0);
        assert_eq!(position(&mut h), 400.0);
        // Scrolled away, the top row no longer takes the tap.
        h.tap([50.0, 75.0]);
        assert_eq!(taps(&mut h), 1);
        scroll(&mut h, -1000.0);
        assert_eq!(position(&mut h), 0.0);
    }

    #[test]
    fn test_scroll_ignores_wheel_outside() {
        let ui = column();
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        h.event(&Event::Scroll {
            position: [500.0, 50.0].into(),
            delta: [0.0, 30.0].into(),
        });
        assert_eq!(position(&mut h), 0.0);
    }
}
