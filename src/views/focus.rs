use crate::*;
use std::any::Any;

/// Struct for the `focus` modifier.
#[derive(Clone)]
pub struct Focus<F> {
    func: F,
}

impl<V, F> DynView for Focus<F>
where
    V: View,
    F: Fn(bool) -> V + Clone + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchBegin { id: _, position } => {
                if self.hittest(vid, *position, cx).is_some() {
                    cx.focused_id = Some(vid);
                    cx.set_dirty();
                }
            }
            Event::Key(Key::Escape) if cx.focused_id == Some(vid) => {
                cx.focused_id = None;
                cx.set_dirty();
            }
            _ => (),
        }
        let cid = cx.child(vid, 0);
        (self.func)(Some(vid) == cx.focused_id).process(event, cid, cx, actions);
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let id = vid;
        let cid = args.cx.child(vid, 0);
        (self.func)(Some(id) == args.cx.focused_id).draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let id = vid;
        let cid = args.cx.child(vid, 0);

        (self.func)(Some(id) == args.cx.focused_id).layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let id = vid;
        let cid = cx.child(vid, 0);
        (self.func)(Some(id) == cx.focused_id).dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let id = vid;
        let cid = cx.child(vid, 0);

        (self.func)(Some(id) == cx.focused_id).hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let id = vid;
        let cid = cx.child(vid, 0);
        (self.func)(Some(id) == cx.focused_id).commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let id = vid;
        let cid = cx.child(vid, 0);

        (self.func)(Some(id) == cx.focused_id).access(cid, cx, nodes)
    }
}

impl<F> private::Sealed for Focus<F> {}

/// Calls calls a function with true if the view subtree returned
/// by the function has the keyboard focus.
pub fn focus<V: View, F: Fn(bool) -> V + Clone + 'static>(f: F) -> impl View {
    Focus { func: f }
}
