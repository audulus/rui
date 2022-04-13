use crate::*;
use std::any::TypeId;

/// Struct for `any_view`
pub struct AnyView {
    child: Box<dyn View>,
}

impl AnyView {

    pub fn new(child: impl View + 'static) -> Self {
        Self { child: Box::new(child) }
    }

    fn id(&self) -> TypeId {
        self.child.tid()
    }
}

impl View for AnyView
{
    fn tid(&self) -> TypeId {
        self.child.tid()
    }

    fn print(&self, id: ViewID, cx: &mut Context) {
        println!("AnyView {{");
        (self.child).print(id.child(&self.id()), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.process(&event, id.child(&self.id()), cx, vger);
    }

    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER) {
        self.child.draw(id.child(&self.id()), cx, vger);
    }

    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize {
        self.child.layout(id.child(&self.id()), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID> {
        self.child.hittest(id.child(&self.id()), pt, cx, vger)
    }

    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&self.id()), cx, cmds)
    }

    fn gc(&self, id: ViewID, cx: &mut Context, map: &mut Vec<ViewID>) {
        self.child.gc(id.child(&self.id()), cx, map)
    }

    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&self.id()), cx, nodes)
    }
}

/// Switches between views according to a boolean.
pub fn any_view(view: impl View) -> AnyView {
    AnyView {
        child: Box::new(view)
    }
}

impl private::Sealed for AnyView {}

#[cfg(test)]
mod tests {

    use super::*;
    
    #[test]
    fn test_typeid() {
        let b: Box<dyn View> = Box::new(EmptyView{});
        let tid = b.tid();
        println!("{:?}", tid);
        assert_eq!(tid, TypeId::of::<EmptyView>());
    }

    #[test]
    fn test_typeid2() {
        let a = EmptyView{};
        let b = rectangle();
        assert_ne!(a.tid(), b.tid());
    }

    #[test]
    fn test_typeid3() {
        let a = any_view(EmptyView{});
        let b = any_view(rectangle());
        assert_ne!(a.tid(), b.tid());
    }
}

