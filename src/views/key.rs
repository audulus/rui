use crate::*;
use std::any::Any;

/// Describes if the KeyView action should trigger when pressing or releasing a key
#[derive(Clone)]
pub enum KeyViewKind {
    Pressed,
    Released,
}

/// Struct for the `key` modifier.
#[derive(Clone)]
pub struct KeyView<V, F> {
    child: V,
    func: F,
    kind: KeyViewKind,
}

impl<V, F, A> KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
{
    pub fn new_pressed(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Pressed,
        }
    }

    pub fn new_released(v: V, f: F) -> Self {
        KeyView {
            child: v,
            func: f,
            kind: KeyViewKind::Released,
        }
    }
}

impl<V, F, A> DynView for KeyView<V, F>
where
    V: View,
    F: Fn(&mut Context, Key) -> A + Clone + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match self.kind {
            KeyViewKind::Pressed => {
                if let Event::Key(key) = &event {
                    actions.push(Box::new((self.func)(cx, *key)));
                } else {
                    let cid = cx.child(vid, 0);
                    self.child.process(event, cid, cx, actions);
                }
            }
            KeyViewKind::Released => {
                if let Event::KeyReleased(key) = &event {
                    actions.push(Box::new((self.func)(cx, *key)));
                } else {
                    let cid = cx.child(vid, 0);
                    self.child.process(event, cid, cx, actions);
                }
            }
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
    }

    fn dirty(&self, vid: ViewId, xform: LocalToWorld, cx: &mut Context) {
        let cid = cx.child(vid, 0);
        self.child.dirty(cid, xform, cx);
    }

    fn hittest(&self, vid: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        let cid = cx.child(vid, 0);

        self.child.hittest(cid, pt, cx)
    }

    fn commands(&self, vid: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        let cid = cx.child(vid, 0);
        self.child.commands(cid, cx, cmds);
    }

    fn access(
        &self,
        vid: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        let cid = cx.child(vid, 0);

        self.child.access(cid, cx, nodes)
    }
}

impl<V, F> private::Sealed for KeyView<V, F> {}
