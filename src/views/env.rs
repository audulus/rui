use crate::*;
use std::any::Any;

#[derive(Clone)]
struct EnvView<S, V, F> {
    func: F,
    phantom_s: std::marker::PhantomData<S>,
    phantom_v: std::marker::PhantomData<V>,
}

impl<S, V, F> DynView for EnvView<S, V, F>
where
    V: View,
    S: Clone + Default + 'static,
    F: Fn(S, &mut Context) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let cid = cx.child(vid, 0);
        (self.func)(cx.init_env(&S::default), cx).process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        (self.func)(args.cx.init_env(&S::default), args.cx).draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        (self.func)(args.cx.init_env(&S::default), args.cx).layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        (self.func)(cx.init_env(&S::default), cx).dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        (self.func)(cx.init_env(&S::default), cx).hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        (self.func)(cx.init_env(&S::default), cx).commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        (self.func)(cx.init_env(&S::default), cx).access(cid, cx, nodes)
    }
}

impl<S, V, F> private::Sealed for EnvView<S, V, F> {}

/// Reads from the environment.
pub fn env<S: Clone + Default + 'static, V: View, F: Fn(S, &mut Context) -> V + Clone + 'static>(
    f: F,
) -> impl View {
    EnvView {
        func: f,
        phantom_s: Default::default(),
        phantom_v: Default::default(),
    }
}

/// Struct for the `env` modifier.
#[derive(Clone)]
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

impl<V, E> DynView for SetenvView<V, E>
where
    V: View,
    E: Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let old = cx.set_env(&self.env_val);
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
        old.and_then(|s| cx.set_env(&s));
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let old = args.cx.set_env(&self.env_val);
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
        old.and_then(|s| args.cx.set_env(&s));
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let old = args.cx.set_env(&self.env_val);
        let cid = args.cx.child(vid, 0);
        let sz = self.child.layout(cid, args);
        old.and_then(|s| args.cx.set_env(&s));
        sz
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let old = cx.set_env(&self.env_val);
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform, cx);
        old.and_then(|s| cx.set_env(&s));
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let old = cx.set_env(&self.env_val);
        let cid = cx.child(vid, 0);
        let r = self.child.hittest(cid, pt, cx);
        old.and_then(|s| cx.set_env(&s));
        r
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let old = cx.set_env(&self.env_val);
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
        old.and_then(|s| cx.set_env(&s));
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let old = cx.set_env(&self.env_val);
        let cid = cx.child(vid, 0);
        let r = self.child.access(cid, cx, nodes);
        old.and_then(|s| cx.set_env(&s));
        r
    }
}

impl<V, E> private::Sealed for SetenvView<V, E> {}
