use crate::*;
use std::any::Any;

/// Struct for the `Hover` gesture.
pub struct Hover<V, F> {
    child: V,
    func: F,
}

impl<V, F, A> Hover<V, F>
where
    V: View,
    F: Fn(&mut Context, bool) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F, A> View for Hover<V, F>
where
    V: View,
    F: Fn(&mut Context, bool) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchMove { id: _, position } => {
                if cx.mouse_button.is_none() {
                    let inside = self.hittest(vid, *position, cx).is_some();
                    actions.push(Box::new((self.func)(cx, inside)));
                }
            }
            _ => (),
        }
        self.child.process(event, vid.child(&0), cx, actions)
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.child.draw(id.child(&0), args)
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&0), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx)
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
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> private::Sealed for Hover<V, F> {}
