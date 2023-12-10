use crate::*;
use std::any::Any;

pub trait TapFn<A> {
    fn call(&self, cx: &mut Context, pt: LocalPoint, button: Option<MouseButton>) -> A;
}

pub struct TapFunc<F> {
    pub f: F
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A> TapFn<A> for TapFunc<F> {
    fn call(&self, cx: &mut Context, pt: LocalPoint, button: Option<MouseButton>) -> A {
        (self.f)(cx, pt, button)
    }
}

pub struct TapAdapter<F> {
    pub f: F
}

impl<A: 'static, F: Fn(&mut Context) -> A> TapFn<A> for TapAdapter<F> {
    fn call(&self, cx: &mut Context, _pt: LocalPoint, _button: Option<MouseButton>) -> A {
        (self.f)(cx)
    }
}

/// Struct for the `tap` gesture.
pub struct TapP<V: View, F, A> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,

    phantom_a: std::marker::PhantomData<A>
}

impl<V, F, A> TapP<V, F, A>
where
    V: View,
    F: TapFn<A> + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f, phantom_a: std::marker::PhantomData::default() }
    }
}

impl<V, F, A> View for TapP<V, F, A>
where
    V: View,
    F: TapFn<A> + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        let vid = cx.view_id(path);
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(path, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    actions.push(Box::new(self.func.call(cx, *position, cx.mouse_button)))
                }
            }
            _ => (),
        }
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

impl<V, F, A> private::Sealed for TapP<V, F, A> where V: View {}
