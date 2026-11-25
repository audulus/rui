use crate::*;
use std::any::Any;

/// Weak reference to app state.
///
/// To get the underlying value, you'll need a `Context`, which is passed
/// to all event handlers, and functions passed to `state`.
pub struct StateHandle<S> {
    pub(crate) id: ViewId,
    phantom: std::marker::PhantomData<S>,
}

impl<S> Copy for StateHandle<S> {}

impl<S> Clone for StateHandle<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static> StateHandle<S> {
    pub fn new(id: ViewId) -> Self {
        Self {
            id,
            phantom: Default::default(),
        }
    }

    /// Makes it convenient to get a function to set the value.
    pub fn setter(self) -> impl Fn(S, &mut Context) {
        move |s, cx| cx[self] = s
    }
}

impl<S: 'static> Binding<S> for StateHandle<S> {
    fn get<'a>(&self, cx: &'a Context) -> &'a S {
        cx.get(*self)
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        cx.get_mut(*self)
    }
}

#[derive(Clone)]
pub struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> DynView for StateView<D, F>
where
    V: View,
    S: 'static,
    D: Fn() -> S + Clone + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        cx.init_state(vid, &self.default);
        if let Event::Anim = event
            && cx.skip_anim(vid)
        {
            return;
        }
        let before = cx.enter_state(vid);
        let cid = cx.child(vid, 0);
        (self.func)(StateHandle::new(vid), cx).process(event, cid, cx, actions);
        cx.leave_state(before);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        args.cx.init_state(vid, &self.default);
        let cid = args.cx.child(vid, 0);
        (self.func)(StateHandle::new(vid), args.cx).draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        args.cx.init_state(vid, &self.default);

        // Reuse the last layout unless the proposed size changed, or a state it depends on
        // was written: the state of an enclosing `state` view, or any state at or under this
        // one.
        if let Some(size) = args.cx.cached_layout(vid, args.sz) {
            return size;
        }

        let stamp = args.cx.write_gen;
        let proposed = args.sz;
        let before = args.cx.enter_state(vid);

        let view = (self.func)(StateHandle::new(vid), args.cx);
        let cid = args.cx.child(vid, 0);
        let child_size = view.layout(cid, args);

        args.cx.leave_state(before);

        let layout_box = LayoutBox {
            rect: LocalRect::new(LocalPoint::zero(), child_size),
            offset: LocalOffset::zero(),
        };
        args.cx.update_layout(vid, layout_box);
        args.cx.store_layout(vid, stamp, proposed, child_size);

        child_size
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        cx.init_state(vid, &self.default);

        if cx.is_dirty(vid) {
            // Add a region.
            let rect = cx.get_layout(vid).rect;
            let pts: [LocalPoint; 4] = [
                rect.min(),
                [rect.max_x(), rect.min_y()].into(),
                [rect.min_x(), rect.max_y()].into(),
                rect.max(),
            ];
            let world_pts = pts.map(|p| xform.transform_point(p));
            cx.dirty_region.add_rect(WorldRect::from_points(world_pts));
        } else if cx.is_subtree_dirty(vid) {
            let cid = cx.child(vid, 0);
            (self.func)(StateHandle::new(vid), cx).dirty(cid, xform, cx);
        }
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        cx.init_state(vid, &self.default);
        let cid = cx.child(vid, 0);
        (self.func)(StateHandle::new(vid), cx).hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.init_state(vid, &self.default);
        let cid = cx.child(vid, 0);
        (self.func)(StateHandle::new(vid), cx).commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        cx.init_state(vid, &self.default);

        // The nodes under this view were sent in an earlier update and nothing they depend on
        // changed since.
        if let Some(id) = cx.cached_access(vid) {
            return id;
        }

        let stamp = cx.write_gen;
        let before = cx.enter_state(vid);
        let cid = cx.child(vid, 0);
        let id = (self.func)(StateHandle::new(vid), cx).access(cid, cx, nodes);
        cx.leave_state(before);
        cx.store_access(vid, stamp, id);
        id
    }
}

impl<S, F> private::Sealed for StateView<S, F> {}

/// State allows you to associate some state with a view.
/// This is what you'll use for a data model, as well as per-view state.
/// Your state should be efficiently clonable. Use Rc as necessary.
///
/// `initial` is the initial value for your state.
///
/// `f` callback which is passed a `State<S>`
pub fn state<
    S: 'static,
    V: View,
    D: Fn() -> S + Clone + 'static,
    F: Fn(StateHandle<S>, &Context) -> V + Clone + 'static,
>(
    initial: D,
    f: F,
) -> StateView<D, F> {
    StateView {
        default: initial,
        func: f,
    }
}

/// Convenience to get the context.
pub fn with_cx<V: View, F: Fn(&Context) -> V + Clone + 'static>(f: F) -> impl View {
    state(|| (), move |_, cx| f(cx))
}

/// Convenience to retreive a reference to a value in the context.
pub fn with_ref<V: View, F: Fn(&T) -> V + Clone + 'static, T>(
    binding: impl Binding<T>,
    f: F,
) -> impl View {
    with_cx(move |cx| f(binding.get(cx)))
}
