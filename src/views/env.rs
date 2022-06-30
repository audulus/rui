use crate::*;
use std::any::Any;

struct EnvView<S, V, F> {
    func: F,
    phantom_s: std::marker::PhantomData<S>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<S, V, F> View for EnvView<S, V, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
{
    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger, actions: &mut Vec<Box<dyn Any>>) {
        (self.func)(cx.init_env(&S::default), cx).process(event, id.child(&0), cx, vger, actions);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        (self.func)(cx.init_env(&S::default), cx).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        let child_size =
            (self.func)(cx.init_env(&S::default), cx).layout(id.child(&0), sz, cx, vger);

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), child_size),
                offset: LocalOffset::zero(),
            },
        );

        child_size
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        (self.func)(cx.init_env(&S::default), cx).dirty(id.child(&0), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        (self.func)(cx.init_env(&S::default), cx).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(cx.init_env(&S::default), cx).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(id);
        (self.func)(cx.init_env(&S::default), cx).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(cx.init_env(&S::default), cx).access(id.child(&0), cx, nodes)
    }
}

impl<S, V, F> private::Sealed for EnvView<S, V, F> {}

/// Reads from the environment.
pub fn env<S: Clone + Default + 'static, V: View, F: Fn(S, &mut Context) -> V + 'static>(
    f: F,
) -> impl View {
    EnvView {
        func: f,
        phantom_s: Default::default(),
        phantom_v: Default::default(),
    }
}

/// Struct for the `env` modifier.
pub struct SetenvView<V, E> {
    child: V,
    env_val: E,
}

impl<V, E> SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    pub fn new(child: V, env_val: E) -> Self {
        Self { child, env_val }
    }
}

impl<V, E> View for SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger, actions: &mut Vec<Box<dyn Any>>) {
        let old = cx.set_env(&self.env_val);
        self.child.process(event, id.child(&0), cx, vger, actions);
        old.and_then(|s| cx.set_env(&s));
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        let old = cx.set_env(&self.env_val);
        self.child.draw(id.child(&0), cx, vger);
        old.and_then(|s| cx.set_env(&s));
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        let old = cx.set_env(&self.env_val);
        let sz = self.child.layout(id.child(&0), sz, cx, vger);
        old.and_then(|s| cx.set_env(&s));
        sz
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let old = cx.set_env(&self.env_val);
        self.child.dirty(id.child(&0), xform, cx);
        old.and_then(|s| cx.set_env(&s));
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        let old = cx.set_env(&self.env_val);
        let r = self.child.hittest(id.child(&0), pt, cx, vger);
        old.and_then(|s| cx.set_env(&s));
        r
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let old = cx.set_env(&self.env_val);
        self.child.commands(id.child(&0), cx, cmds);
        old.and_then(|s| cx.set_env(&s));
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        let old = cx.set_env(&self.env_val);
        self.child.gc(id.child(&0), cx, map);
        old.and_then(|s| cx.set_env(&s));
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        let old = cx.set_env(&self.env_val);
        let r = self.child.access(id.child(&0), cx, nodes);
        old.and_then(|s| cx.set_env(&s));
        r
    }
}

impl<V, E> private::Sealed for SetenvView<V, E> {}
