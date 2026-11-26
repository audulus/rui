use crate::*;
use std::any::Any;

pub trait HoverFn: Clone {
    fn call(&self, cx: &mut Context, pt: LocalPoint, inside: bool, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct HoverFuncP<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint) -> A + Clone + 'static> HoverFn for HoverFuncP<F> {
    fn call(
        &self,
        cx: &mut Context,
        pt: LocalPoint,
        inside: bool,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if inside {
            actions.push(Box::new((self.f)(cx, pt)))
        }
    }
}

#[derive(Clone)]
pub struct HoverFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, bool) -> A + Clone + 'static> HoverFn for HoverFunc<F> {
    fn call(
        &self,
        cx: &mut Context,
        _pt: LocalPoint,
        inside: bool,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(cx, inside)))
    }
}

/// Struct for the `hover` and 'hover_p` gestures.
#[derive(Clone)]
pub struct Hover<V, F> {
    child: V,
    func: F,
}

impl<V, F> Hover<V, F>
where
    V: View,
    F: HoverFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Hover<V, F>
where
    V: View,
    F: HoverFn + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::TouchMove { position, .. } = &event
            && cx.mouse_button.is_none()
        {
            let inside = self.hittest(vid, *position, cx).is_some();
            self.func.call(cx, *position, inside, actions);
        }
        if let Event::TouchEnd { position, .. } = &event {
            let inside = self.hittest(vid, *position, cx).is_some();
            self.func.call(cx, *position, inside, actions);
        }
        let cid = cx.child(vid, 0);
        self.child.process(event, cid, cx, actions);
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

impl<V, F> private::Sealed for Hover<V, F> {}
