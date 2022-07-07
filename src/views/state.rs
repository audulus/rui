use crate::*;
use std::any::Any;

/// Weak reference to app state.
pub struct State<S> {
    pub(crate) id: ViewId,
    phantom: std::marker::PhantomData<S>,
}

impl<S> Copy for State<S> {}

impl<S> Clone for State<S> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<S: 'static> State<S> {
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

impl<S: 'static> Binding<S> for State<S> {
    fn get<'a>(&self, cx: &'a mut Context) -> &'a S {
        cx.get(*self)
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        cx.get_mut(*self)
    }
}

struct StateView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for StateView<D, F>
where
    V: View,
    S: 'static,
    D: Fn() -> S + 'static,
    F: Fn(State<S>, &mut Context) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        cx.init_state(id, &self.default);
        (self.func)(State::new(id), cx).process(event, id.child(&0), cx, vger, actions);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        cx.init_state(id, &self.default);
        (self.func)(State::new(id), cx).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        cx.init_state(id, &self.default);

        // Do we need to recompute layout?
        let mut compute_layout = true;

        if let Some(deps) = (cx.deps.get(&id)).clone() {
            let mut any_dirty = false;
            for dep in deps {
                if let Some(holder) = cx.state_map.get_mut(&dep) {
                    if holder.dirty {
                        any_dirty = true;
                        break;
                    }
                }
            }

            compute_layout = any_dirty;
        }

        if compute_layout {
            cx.id_stack.push(id);

            let view = (self.func)(State::new(id), cx);

            let child_size = view.layout(id.child(&0), sz, cx, vger);

            // Compute layout dependencies.
            let mut deps = vec![];
            deps.append(&mut cx.id_stack.clone());
            view.gc(id.child(&0), cx, &mut deps);

            cx.deps.insert(id, deps);

            cx.layout.insert(
                id,
                LayoutBox {
                    rect: LocalRect::new(LocalPoint::zero(), child_size),
                    offset: LocalOffset::zero(),
                },
            );

            cx.id_stack.pop();
        }

        cx.layout[&id].rect.size
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let default = &self.default;
        let holder = cx.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((default)()),
            dirty: false,
        });

        if holder.dirty {
            // Add a region.
            let rect = cx.layout[&id].rect;
            let pts: [LocalPoint; 4] = [
                rect.min(),
                [rect.max_x(), rect.min_y()].into(),
                [rect.min_x(), rect.max_y()].into(),
                rect.max(),
            ];
            let world_pts = pts.map(|p| xform.transform_point(p));
            cx.dirty_region.add_rect(WorldRect::from_points(world_pts));
        } else {
            (self.func)(State::new(id), cx).dirty(id.child(&0), xform, cx);
        }
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        cx.init_state(id, &self.default);
        (self.func)(State::new(id), cx).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.init_state(id, &self.default);
        (self.func)(State::new(id), cx).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        cx.init_state(id, &self.default);
        map.push(id);
        (self.func)(State::new(id), cx).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        cx.init_state(id, &self.default);
        (self.func)(State::new(id), cx).access(id.child(&0), cx, nodes)
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
    D: Fn() -> S + 'static,
    F: Fn(State<S>, &mut Context) -> V + 'static,
>(
    initial: D,
    f: F,
) -> impl View {
    StateView {
        default: initial,
        func: f,
    }
}

/// Convenience to get the context.
pub fn get_cx<V: View, F: Fn(&mut Context) -> V + 'static>(f: F) -> impl View {
    state(|| (), move |_, cx| f(cx))
}

struct StateView2<'a, S, V, DefaultFn, F, OuterData>
where
    S: 'static,
    F: Fn(&S) -> V + 'a,
{
    default: DefaultFn,
    func: F,
    phantom: std::marker::PhantomData<fn() -> (OuterData, &'a i32, S)>,
}

impl<'a, S, V, DefaultFn, F, Data> View2<Data> for StateView2<'a, S, V, DefaultFn, F, Data>
where
    V: View2<S>,
    S: 'static,
    Data: 'static,
    DefaultFn: Fn() -> S + 'static,
    F: Fn(&S) -> V + 'a,
{
    type State = (Option<S>, V::State);

    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        _data: &mut Data,
    ) {
        let s = state.0.get_or_insert_with(|| (self.default)());
        let v = (self.func)(s);
        v.process(event, id.child(&0), cx, vger, &mut state.1, s);
    }

    fn draw(
        &self,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        _data: &Data,
    ) {
        let s = state.0.get_or_insert_with(|| (self.default)());
        let v = (self.func)(s);
        v.draw(id.child(&0), cx, vger, &mut state.1, s);
    }

    fn layout(
        &self,
        id: ViewId,
        sz: LocalSize,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        _data: &Data,
    ) -> LocalSize {
        // Do we need to recompute layout?
        let mut compute_layout = true;

        if let Some(deps) = (cx.deps.get(&id)).clone() {
            let mut any_dirty = false;
            for dep in deps {
                if let Some(holder) = cx.state_map.get_mut(&dep) {
                    if holder.dirty {
                        any_dirty = true;
                        break;
                    }
                }
            }

            compute_layout = any_dirty;
        }

        if compute_layout {
            cx.id_stack.push(id);

            let s = state.0.get_or_insert_with(|| (self.default)());
            let v = (self.func)(s);

            let child_size = v.layout(id.child(&0), sz, cx, vger, &mut state.1, s);

            // Compute layout dependencies.
            let mut deps = vec![];
            deps.append(&mut cx.id_stack.clone());
            // view.gc(id.child(&0), cx, &mut deps);

            cx.deps.insert(id, deps);

            cx.layout.insert(
                id,
                LayoutBox {
                    rect: LocalRect::new(LocalPoint::zero(), child_size),
                    offset: LocalOffset::zero(),
                },
            );

            cx.id_stack.pop();
        }

        cx.layout[&id].rect.size
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        _data: &Data,
    ) -> Option<ViewId> {
        let s = state.0.get_or_insert_with(|| (self.default)());
        let v = (self.func)(s);
        v.hittest(id.child(&0), pt, cx, vger, &mut state.1, s)
    }
}

pub fn state2<
    'a,
    S: 'static,
    Data: 'static,
    V: View2<S> + 'a,
    D: Fn() -> S + 'static,
    F: Fn(&S) -> V + 'a,
>(
    initial: D,
    f: F,
) -> impl View2<Data> + 'a {
    StateView2::<'a, S, V, D, F, Data> {
        default: initial,
        func: f,
        phantom: Default::default(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn gui_func(_view: impl View2<()>) {}

    #[test]
    fn test_state2() {
        gui_func(state2(|| 1, |_| empty_view2()));
    }

    trait View {
        fn draw(&self);
    }

    struct State<F> {
        f: F,
    }

    impl<F> View for State<F>
    where
        F: for<'a> Fn(&'a String) -> Box<dyn View + 'a>,
    {
        fn draw(&self) {
            // Get the state from somewhere.
            let s = "hello world".to_string();
            (self.f)(&s).draw();
        }
    }

    struct Empty {}

    impl View for Empty {
        fn draw(&self) {}
    }

    fn state<'b, F: 'b + for<'a> Fn(&'a String) -> Box<dyn View + 'a>>(f: F) -> Box<dyn View + 'b> {
        Box::new(State { f })
    }

    fn empty_view() -> Box<dyn View> {
        Box::new(Empty {})
    }

    fn my_ui<'a>(x: &'a String) -> Box<dyn View + 'a> {
        state(move |y| {
            println!("{} {}", x, y);
            empty_view()
        })
    }

    #[test]
    fn test_state_nested() {
        state(move |x| {
            my_ui(x);
            state(move |y| {
                println!("{} {}", x, y);
                empty_view()
            })
        })
        .draw();
    }
}
