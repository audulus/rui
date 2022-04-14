use crate::*;
use std::any::TypeId;

/// Trait for the unit of UI composition.
pub trait View: private::Sealed + 'static {
    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// Prints a description of the view for debugging.
    fn print(&self, id: ViewId, cx: &mut Context);

    /// Processes an event.
    fn process(&self, event: &Event, id: ViewId, cx: &mut Context, vger: &mut VGER);

    /// Draws the view using vger.
    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut VGER);

    /// Lays out subviews and return the size of the view.
    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut VGER) -> LocalSize;

    /// Returns the topmost view which the point intersects.
    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut VGER,
    ) -> Option<ViewId>;

    /// Accumulates information about menu bar commands.
    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>);

    // /// Copies state currently in use to a new StateMap (the rest are dropped).
    fn gc(&self, _id: ViewId, _cx: &mut Context, _map: &mut Vec<ViewId>);

    /// Builds an AccessKit tree. The node ID for the subtree is returned. All generated nodes are accumulated.
    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId>;
}

pub struct EmptyView {}

impl View for EmptyView {
    fn print(&self, _id: ViewId, _cx: &mut Context) {
        println!("EmptyView");
    }
    fn process(&self, _event: &Event, _id: ViewId, _cx: &mut Context, _vger: &mut VGER) {}
    fn draw(&self, _id: ViewId, _cx: &mut Context, _vger: &mut VGER) {}
    fn layout(
        &self,
        _id: ViewId,
        _sz: LocalSize,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> LocalSize {
        [0.0, 0.0].into()
    }
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut VGER,
    ) -> Option<ViewId> {
        None
    }

    fn commands(&self, _id: ViewId, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    fn gc(&self, _id: ViewId, _cx: &mut Context, _map: &mut Vec<ViewId>) {}

    fn access(
        &self,
        _id: ViewId,
        _cx: &mut Context,
        _nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        None
    }
}

impl private::Sealed for EmptyView {}
