use crate::*;

/// Struct for the `window_title` modifier.
pub struct TitleView<V> {
    child: V,
    title: String,
}

impl<V> TitleView<V>
where
    V: View,
{
    pub fn new(v: V, title: &str) -> Self {
        Self {
            child: v,
            title: title.into(),
        }
    }
}

impl<V> View for TitleView<V>
where
    V: View,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        (self.child).print(id.child(&0), cx);
        println!(".window_title()");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger);
        if cx.window_title != self.title {
            cx.window_title = self.title.clone();
            cx.window.as_ref().unwrap().set_title(&self.title)
        }
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V> private::Sealed for TitleView<V> {}

/// Struct for the `fullscreen` modifier.
pub struct FullscreenView<V> {
    child: V,
}

impl<V> FullscreenView<V>
where
    V: View,
{
    pub fn new(v: V) -> Self {
        Self { child: v }
    }
}

impl<V> View for FullscreenView<V>
where
    V: View,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        (self.child).print(id.child(&0), cx);
        println!(".fullscreen()");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.process(event, id.child(&0), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger);
        cx.window
            .as_ref()
            .unwrap()
            .set_fullscreen(Some(tao::window::Fullscreen::Borderless(None)))
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V> private::Sealed for FullscreenView<V> {}
