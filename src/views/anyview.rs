use crate::*;
use std::any::Any;
use std::any::TypeId;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

    fn id_hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.id().hash(&mut hasher);
        hasher.finish()
    }
}

impl View for AnyView {
    fn tid(&self) -> TypeId {
        self.child.tid()
    }

    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        path.push(self.id_hash());
        self.child.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(self.id_hash());
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(self.id_hash());
        let sz = self.child.layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(self.id_hash());
        self.child.dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(self.id_hash());
        let vid = self.child.hittest(path, pt, cx);
        path.pop();
        vid
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(self.id_hash());
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(self.id_hash());
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(self.id_hash());
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
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
