use crate::*;
use std::any::TypeId;

/// Trait for the unit of UI composition.
pub trait View: private::Sealed + 'static {

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Prints a description of the view for debugging.
    fn print(&self, id: ViewID, cx: &mut Context);

    /// Processes an event.
    fn process(&self, event: &Event, id: ViewID, cx: &mut Context, vger: &mut VGER);

    /// Draws the view using vger.
    fn draw(&self, id: ViewID, cx: &mut Context, vger: &mut VGER);

    /// Lays out subviews and return the size of the view.
    fn layout(&self, id: ViewID, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize;

    /// Returns the topmost view which the point intersects.
    fn hittest(
        &self,
        id: ViewID,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewID>;

    /// Accumulates information about menu bar commands.
    fn commands(&self, id: ViewID, cx: &mut Context, cmds: &mut Vec<CommandInfo>);

    // /// Copies state currently in use to a new StateMap (the rest are dropped).
    fn gc(&self, _id: ViewID, _cx: &mut Context, _map: &mut Vec<ViewID>);

    /// Builds an AccessKit tree. The node ID for the subtree is returned. All generated nodes are accumulated.
    fn access(
        &self,
        id: ViewID,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId>;
}

pub struct EmptyView {}

impl View for EmptyView {
    fn print(&self, _id: ViewID, _cx: &mut Context) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event, _id: ViewID, _cx: &mut Context, _vger: &mut VGER) {}
    fn draw(&self, _id: ViewID, _cx: &mut Context, _vger: &mut VGER) {}
    fn layout(
        &self,
        _id: ViewID,
        _sz: LocalSize,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> LocalSize {
        [0.0, 0.0].into()
    }
    fn hittest(
        &self,
        _id: ViewID,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewID> {
        None
    }

    fn commands(&self, _id: ViewID, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    fn gc(&self, _id: ViewID, _cx: &mut Context, _map: &mut Vec<ViewID>) {}

    fn access(
        &self,
        _id: ViewID,
        _cx: &mut Context,
        _nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        None
    }
}

impl private::Sealed for EmptyView {}
