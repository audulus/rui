use crate::*;
use std::any::TypeId;

/// Trait for the unit of UI composition.
pub trait View: private::Sealed + 'static {
    /// Builds an AccessKit tree. The node ID for the subtree is returned. All generated nodes are accumulated.
    fn access(
        &self,
        _id: ViewId,
        _cx: &mut Context,
        _nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        None
    }

    /// Accumulates information about menu bar commands.
    fn commands(&self, _id: ViewId, _cx: &mut Context, _cmds: &mut Vec<CommandInfo>) {}

    /// Determines dirty regions which need repainting.
    fn dirty(&self, _id: ViewId, _xform: LocalToWorld, _cx: &mut Context) {}

    /// Draws the view using vger.
    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger);

    /// Gets IDs for state currently in use.
    fn gc(&self, _id: ViewId, _cx: &mut Context, _map: &mut Vec<ViewId>) {}

    /// Returns the topmost view which the point intersects.
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        None
    }

    /// For detecting flexible sized things in stacks.
    fn is_flexible(&self) -> bool {
        false
    }

    /// Lays out subviews and return the size of the view.
    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize;

    /// Prints a description of the view for debugging.
    fn print(&self, id: ViewId, cx: &mut Context);

    /// Processes an event.
    fn process(&self, _event: &Event, _id: ViewId, _cx: &mut Context, _vger: &mut Vger) {}

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

pub trait View2<Data>: 'static {
    /// Draws the view using vger.
    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger);

    /// Lays out subviews and return the size of the view.
    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize;

    /// Processes an event.
    fn process(
        &self,
        _event: &Event,
        _id: ViewId,
        _cx: &mut Context,
        _vger: &mut Vger,
        _data: State<Data>,
    ) {
    }

    /// Returns the topmost view which the point intersects.
    fn hittest(
        &self,
        _id: ViewId,
        _pt: LocalPoint,
        _cx: &mut Context,
        _vger: &mut Vger,
    ) -> Option<ViewId> {
        None
    }
}
