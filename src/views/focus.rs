use crate::*;
use std::any::Any;

/// Struct for the `focus` modifier.
pub struct Focus<F> {
    func: F,
}

impl<V, F> View for Focus<F>
where
    V: View,
    F: Fn(bool) -> V + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchBegin { id: _, position } => {
                if self.hittest(vid, *position, cx).is_some() {
                    cx.focused_id = Some(vid);
                    cx.set_dirty();
                }
            }
            Event::Key(Key::Escape) => {
                if cx.focused_id == Some(vid) {
                    cx.focused_id = None;
                    cx.set_dirty();
                }
            }
            _ => (),
        }
        (self.func)(Some(vid) == cx.focused_id).process(event, vid.child(&0), cx, vger, actions)
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        (self.func)(Some(id) == args.cx.focused_id).draw(id.child(&0), args)
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        (self.func)(Some(id) == args.cx.focused_id).layout(id.child(&0), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        (self.func)(Some(id) == cx.focused_id).dirty(id.child(&0), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
    ) -> Option<ViewId> {
        (self.func)(Some(id) == cx.focused_id).hittest(id.child(&0), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        (self.func)(Some(id) == cx.focused_id).commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        (self.func)(Some(id) == cx.focused_id).gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        (self.func)(Some(id) == cx.focused_id).access(id.child(&0), cx, nodes)
    }
}

impl<F> private::Sealed for Focus<F> {}

/// Calls calls a function with true if the view subtree returned
/// by the function has the keyboard focus.
pub fn focus<V: View, F: Fn(bool) -> V + 'static>(f: F) -> impl View {
    Focus { func: f }
}
