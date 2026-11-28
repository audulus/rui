use crate::*;
use std::any::Any;

/// Struct for the `scroll` view.
#[derive(Clone)]
pub struct ScrollView<V> {
    child: V,
}

impl<V: View> ScrollView<V> {
    /// The scroll position: how far the top of the content is above the top of the view.
    fn position(&self, vid: ViewId, cx: &mut Context) -> StateHandle<f32> {
        let id = vid;
        cx.init_state(id, &|| 0.0f32);
        StateHandle::new(id)
    }

    /// The offset of `content` in a view `height` tall, scrolled to `position`: the content's
    /// top sits at the view's top, moved up by the position (y is up).
    fn content_offset(height: f32, content: LocalSize, position: f32) -> LocalOffset {
        [0.0, height - content.height + position].into()
    }

    /// The viewport and the offset of the content inside it.
    fn geometry(vid: ViewId, cx: &mut Context) -> (LocalRect, LocalOffset) {
        let viewport = cx.get_layout(vid).rect;
        let cid = cx.child(vid, 0);
        let offset = cx.get_layout(cid).offset;
        (viewport, offset)
    }
}

impl<V: View> DynView for ScrollView<V> {
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let (viewport, offset) = Self::geometry(vid, cx);

        let inside = match event {
            Event::TouchBegin { position, .. }
            | Event::TouchMove { position, .. }
            | Event::TouchEnd { position, .. }
            | Event::Scroll { position, .. } => Some(viewport.contains(*position)),
            _ => None,
        };

        // Content that is scrolled out of view takes no presses or wheel events from outside the
        // viewport. Motion outside it reaches the content far away, so hovers end, unless a view
        // is tracking that touch (a drag), which keeps getting the real position.
        let child_event = match (event, inside) {
            (Event::TouchBegin { .. } | Event::Scroll { .. }, Some(false)) => None,
            (Event::TouchMove { id, delta, .. }, Some(false)) if cx.touches[*id].is_default() => {
                Some(Event::TouchMove {
                    id: *id,
                    position: [f32::MIN, f32::MIN].into(),
                    delta: *delta,
                })
            }
            _ => Some(event.offset(-offset)),
        };

        if let Some(child_event) = child_event {
            let cid = cx.child(vid, 0);
            self.child.process(&child_event, cid, cx, actions);
        }

        if let (Event::Scroll { delta, .. }, Some(true)) = (event, inside) {
            let cid = cx.child(vid, 0);
            let content = cx.get_layout(cid).rect.size;
            let max = (content.height - viewport.height()).max(0.0);
            let handle = self.position(vid, cx);
            // The stored position may be past the end if the content got shorter since; start
            // from where the content is shown.
            let new_position = (cx[handle].min(max) + delta.y).clamp(0.0, max);
            if new_position != cx[handle] {
                // Scrolling only moves the content, so it isn't a write: no layout is
                // invalidated, above the view or in its content. Move the content here and ask
                // for a redraw.
                cx.set_state(vid, new_position);
                cx.set_layout_offset(
                    cid,
                    Self::content_offset(viewport.height(), content, new_position),
                );
                cx.set_dirty();
            }
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let (viewport, offset) = Self::geometry(vid, args.cx);
        args.vger.save();
        args.vger.scissor(viewport);
        let visible = args.cx.clip_visible(viewport, args.vger);
        args.vger.translate(offset);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        args.cx.restore_visible(visible);
        args.vger.restore();
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let handle = self.position(vid, args.cx);

        let cid = args.cx.child(vid, 0);
        let content = self.child.layout(cid, args);

        // Keep the position in range when the content or the view changes size.
        let max = (content.height - args.sz.height).max(0.0);
        let position = args.cx[handle].min(max);

        let cid = args.cx.child(vid, 0);
        args.cx.update_layout(
            cid,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), content),
                offset: Self::content_offset(args.sz.height, content, position),
            },
        );

        args.cx.update_layout(
            vid,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), args.sz),
                offset: LocalOffset::zero(),
            },
        );
        args.sz
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let (_, offset) = Self::geometry(vid, cx);
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform.pre_translate(offset), cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let (viewport, offset) = Self::geometry(vid, cx);
        if !viewport.contains(pt) {
            return None;
        }
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt - offset, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
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
        let id = h.cx.id_at(&[0, 0]);
        h.cx[StateHandle::<f32>::new(id)]
    }

    fn taps(h: &mut TestHarness<impl View>) -> i32 {
        let id = h.cx.root_id();
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
    fn test_scrolling_moves_the_content_without_layout() {
        let ui = column();
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let root = h.cx.root_id();
        let wheel = Event::Scroll {
            position: [50.0, 50.0].into(),
            delta: [0.0, 30.0].into(),
        };
        ui.process(&wheel, root, &mut h.cx, &mut vec![]);

        // The content moved up by 30 px at once, and the enclosing `state` view keeps its
        // layout: nothing was written.
        assert_eq!(position(&mut h), 30.0);
        let content = h.cx.id_at(&[0, 0, 0]);
        assert_eq!(h.cx.get_layout(content).offset.y, 100.0 - 500.0 + 30.0);
        assert!(h.cx.cached_layout(root, [100.0, 100.0].into()).is_some());
        assert!(h.cx.dirty);

        // The top row now covers only the top 20 px of the view.
        h.tap([50.0, 85.0]);
        assert_eq!(taps(&mut h), 1);
        h.tap([50.0, 75.0]);
        assert_eq!(taps(&mut h), 1);
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

    #[test]
    fn test_hidden_content_gets_no_wheel_from_outside() {
        // A 50 px gesture area at the top of 500 px of content, in a scroll view over the
        // lower 100 px of a 200 px window.
        let ui = state(
            || 0,
            |wheels, _| {
                vstack((
                    rectangle().size([100.0, 100.0]),
                    scroll(vstack((
                        rectangle()
                            .size([100.0, 50.0])
                            .on_scroll(move |cx, _| cx[wheels] += 1),
                        rectangle().size([100.0, 450.0]),
                    ))),
                ))
            },
        );
        let mut h = TestHarness::new(&ui, [100.0, 200.0]);
        let wheels = |h: &mut TestHarness<_>| {
            let id = h.cx.root_id();
            h.cx[StateHandle::<i32>::new(id)]
        };
        let wheel = |h: &mut TestHarness<_>, y: f32| {
            h.event(&Event::Scroll {
                position: [50.0, y].into(),
                delta: [0.0, 60.0].into(),
            });
            h.layout();
        };
        // Over the gesture area, inside the viewport: it sees the wheel, and the view scrolls
        // it out of sight.
        wheel(&mut h, 75.0);
        assert_eq!(wheels(&mut h), 1);
        // Over the top half of the window, where the gesture area now sits hidden.
        wheel(&mut h, 130.0);
        assert_eq!(wheels(&mut h), 1);
    }

    #[test]
    fn test_drag_keeps_its_position_outside() {
        let ui = state(LocalPoint::zero, |last, _| {
            scroll(
                rectangle()
                    .size([100.0, 500.0])
                    .drag_p(move |cx, pt, _, _| cx[last] = pt),
            )
        });
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // Press inside, then move above the viewport while still pressed.
        h.event(&Event::TouchBegin {
            id: 0,
            position: [50.0, 50.0].into(),
        });
        h.event(&Event::TouchMove {
            id: 0,
            position: [50.0, 150.0].into(),
            delta: [0.0, 100.0].into(),
        });
        let id = h.cx.root_id();
        let last = h.cx[StateHandle::<LocalPoint>::new(id)];
        assert!(last.y > 100.0, "{last:?}");
    }

    #[test]
    fn test_scroll_after_content_shrinks() {
        let ui = state(
            || 500.0f32,
            |height, cx| scroll(rectangle().size([100.0, cx[height]])),
        );
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let scroll_by = |h: &mut TestHarness<_>, dy: f32| {
            h.event(&Event::Scroll {
                position: [50.0, 50.0].into(),
                delta: [0.0, dy].into(),
            });
            h.layout();
        };
        scroll_by(&mut h, 400.0);
        let height = StateHandle::<f32>::new(h.cx.root_id());
        h.cx[height] = 200.0;
        h.layout();
        // At most 100 px down now; one notch up shows a move at once.
        scroll_by(&mut h, -40.0);
        assert_eq!(position(&mut h), 60.0);
    }
}
