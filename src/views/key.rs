use crate::*;
use std::any::Any;

/// Struct for the `key` modifier.
pub struct KeyView<V, F> {
    child: V,
    func: F,
}

impl<V, F, A> KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        KeyView { child: v, func: f }
    }
}

impl<V, F, A> View for KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        _vid: ViewId,
        cx: &mut Context,
        _vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::Key(key) = &event {
            actions.push(Box::new((self.func)(cx, key.clone())));
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger)
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

impl<V, F> private::Sealed for KeyView<V, F> {}
