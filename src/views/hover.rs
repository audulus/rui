use crate::*;
use std::any::Any;

pub trait HoverFn {
    fn call(&self, cx: &mut Context, pt: LocalPoint, inside: bool, actions: &mut Vec<Box<dyn Any>>);
}

pub struct HoverFuncP<F> {
    pub f: F
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint) -> A> HoverFn for HoverFuncP<F> {
    fn call(&self, cx: &mut Context, pt: LocalPoint, inside: bool, actions: &mut Vec<Box<dyn Any>>) {
        if inside {
            actions.push(Box::new((self.f)(cx, pt)))
        }
    }
}

pub struct HoverFunc<F> {
    pub f: F
}

impl<A: 'static, F: Fn(&mut Context, bool) -> A> HoverFn for HoverFunc<F> {
    fn call(&self, cx: &mut Context, _pt: LocalPoint, inside: bool, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx, inside)))
    }
}

/// Struct for the `hover` and 'hover_p` gestures.
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

impl<V, F> View for Hover<V, F>
where
    V: View,
    F: HoverFn + 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        if let Event::TouchMove { position, .. } = &event {
            if cx.mouse_button.is_none() {
                let inside = self.hittest(path, *position, cx).is_some();
                self.func.call(cx, *position, inside, actions);
            }
        }
        path.push(0);
        self.child.process(event, path, cx, actions);
        path.pop();
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        path.push(0);
        self.child.draw(path, args);
        path.pop();
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        path.push(0);
        let sz = self.child.layout(path, args);
        path.pop();
        sz
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        path.push(0);
        self.child.dirty(path, xform, cx);
        path.pop();
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        path.push(0);
        let id = self.child.hittest(path, pt, cx);
        path.pop();
        id
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        path.push(0);
        self.child.commands(path, cx, cmds);
        path.pop();
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        path.push(0);
        self.child.gc(path, cx, map);
        path.pop();
    }

    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        path.push(0);
        let node_id = self.child.access(path, cx, nodes);
        path.pop();
        node_id
    }
}

impl<V, F> private::Sealed for Hover<V, F> {}
