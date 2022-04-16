use crate::*;

struct EnvView<D, F> {
    default: D,
    func: F,
}

impl<S, V, D, F> View for EnvView<D, F>
where
    V: View,
    S: Clone + 'static,
    D: Fn() -> S + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        (self.func)(cx.init_env(&self.default), cx).print(id.child(&0), cx);
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        (self.func)(cx.init_env(&self.default), cx).process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        (self.func)(cx.init_env(&self.default), cx).draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        let child_size = (self.func)(cx.init_env(&self.default), cx).layout(id.child(&0), sz, cx, vger);

        cx.layout.insert(
            id,
            LayoutBox {
                rect: LocalRect::new(LocalPoint::zero(), child_size),
                offset: LocalOffset::zero(),
            },
        );

        child_size
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
        region: &mut Region<WorldSpace>,
    ) {
        (self.func)(cx.init_env(&self.default), cx).dirty(id.child(&0), xform, cx, region);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        (self.func)(cx.init_env(&self.default), cx).hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(cx.init_env(&self.default), cx).commands(id.child(&0), cx, cmds);
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        map.push(id);
        (self.func)(cx.init_env(&self.default), cx).gc(id.child(&0), cx, map);
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(cx.init_env(&self.default), cx).access(id.child(&0), cx, nodes)
    }
}

impl<S, F> private::Sealed for EnvView<S, F> {}

/// Reads from the environment.
pub fn env<
    S: Clone + 'static,
    V: View,
    D: Fn() -> S + 'static,
    F: Fn(S, &mut Context) -> V + 'static,
>(
    initial: D,
    f: F,
) -> impl View {
    EnvView {
        default: initial,
        func: f,
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
    E: Clone + 'static
{
    pub fn new(child: V, env_val: E) -> Self {
        Self { child, env_val }
    }
}

impl<V, E> View for SetenvView<V, E>
where
    V: View,
    E: Clone + 'static
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        cx.set_env(&self.env_val);
        (self.child).print(id.child(&0), cx);
        println!(".env()");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        cx.set_env(&self.env_val);
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER) {
        cx.set_env(&self.env_val);
        self.child.draw(id.child(&0), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        cx.set_env(&self.env_val);
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn dirty(
        &self,
        id: ViewId,
        xform: LocalToWorld,
        cx: &mut Context,
        region: &mut Region<WorldSpace>,
    ) {
        cx.set_env(&self.env_val);
        self.child.dirty(id.child(&0), xform, cx, region);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId> {
        cx.set_env(&self.env_val);
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        cx.set_env(&self.env_val);
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        cx.set_env(&self.env_val);
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        cx.set_env(&self.env_val);
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, E> private::Sealed for SetenvView<V, E> {}
