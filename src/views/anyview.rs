use crate::*;
use std::any::Any;
use std::any::TypeId;

/// Struct for `any_view`
pub struct AnyView {
    child: Box<dyn View>,
}

impl AnyView {
    pub fn new(child: impl View) -> Self {
        Self {
            child: Box::new(child),
        }
    }

    fn id(&self) -> TypeId {
        self.child.tid()
    }
}

impl View for AnyView {
    fn tid(&self) -> TypeId {
        self.child.tid()
    }

    fn process(
        &self,
        event: &Event,
        id: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.child.process(event, id.child(&self.id()), cx, actions);
    }

    fn draw(&self, id: ViewId, args: &mut DrawArgs) {
        self.child.draw(id.child(&self.id()), args);
    }

    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize {
        self.child.layout(id.child(&self.id()), args)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&self.id()), xform, cx);
    }

    fn hittest(&self, id: ViewId, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.child.hittest(id.child(&self.id()), pt, cx)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&self.id()), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&self.id()), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&self.id()), cx, nodes)
    }
}

/// Switches between views according to a boolean.
pub fn any_view(view: impl View) -> AnyView {
    AnyView {
        child: Box::new(view),
    }
}

impl private::Sealed for AnyView {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_typeid() {
        let b: Box<dyn View> = Box::new(EmptyView {});
        let tid = b.tid();
        println!("{:?}", tid);
        assert_eq!(tid, TypeId::of::<EmptyView>());
    }

    #[test]
    fn test_typeid2() {
        let a = EmptyView {};
        let b = rectangle();
        assert_ne!(a.tid(), b.tid());
    }

    #[test]
    fn test_typeid3() {
        let a = any_view(EmptyView {});
        let b = any_view(rectangle());
        assert_ne!(a.tid(), b.tid());
    }
}
