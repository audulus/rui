use crate::*;
use std::any::{Any, TypeId};

pub struct DrawArgs<'a> {
    pub cx: &'a mut Context,
    pub vger: &'a mut Vger,
}

pub struct LayoutArgs<'a> {
    pub sz: LocalSize,
    pub cx: &'a mut Context,
    pub text_bounds: &'a mut dyn FnMut(&str, u32, Option<f32>) -> LocalRect,
}

impl<'a> LayoutArgs<'a> {
    pub fn size(&mut self, sz: LocalSize) -> LayoutArgs {
        LayoutArgs {
            sz,
            cx: self.cx,
            text_bounds: self.text_bounds,
        }
    }
}

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
    fn draw(&self, id: ViewId, args: &mut DrawArgs);

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
    ///
    /// `sz` is the available size for the view
    /// `vger` can be used to get text sizing
    ///
    /// Note that we should probably have a separate text
    /// sizing interface so we don't need a GPU and graphics
    /// context set up to test layout.
    fn layout(&self, id: ViewId, args: &mut LayoutArgs) -> LocalSize;

    /// Processes an event.
    fn process(
        &self,
        _event: &Event,
        _id: ViewId,
        _cx: &mut Context,
        _vger: &mut Vger,
        _actions: &mut Vec<Box<dyn Any>>,
    ) {
    }

    /// Returns the type ID of the underlying view.
    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}
