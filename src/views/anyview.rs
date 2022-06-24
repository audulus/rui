use crate::*;
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

    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("AnyView {{");
        (self.child).print(id.child(&self.id()), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.process(event, id.child(&self.id()), cx, vger);
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&self.id()), cx, vger);
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&self.id()), sz, cx, vger)
    }

    fn dirty(&self, id: ViewId, xform: LocalToWorld, cx: &mut Context) {
        self.child.dirty(id.child(&self.id()), xform, cx);
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&self.id()), pt, cx, vger)
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
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&self.id()), cx, nodes)
    }
}

/// Erases the underlying view type, allowing 
/// the function to return multiple different view types
/// 
/// Usage:
/// ```no_run
/// use rui::*;
/// 
/// fn main() {
///     rui(list(vec![7, 42], |i| {
///         if *i == 7 {
///             any_view(circle())
///         } else {
///             any_view(rectangle())
///         }
///         .padding(Auto)
///     }));
/// }
/// ```
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
