use crate::*;
use std::any::Any;

pub enum TouchState {
    Begin,
    End,
}

pub struct TouchInfo {
    pub pt: LocalPoint,
    pub button: Option<MouseButton>,
    pub state: TouchState,
}

pub trait TouchFn: Clone {
    fn call(&self, cx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct TouchFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, TouchInfo) -> A + Clone> TouchFn for TouchFunc<F> {
    fn call(&self, cx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx, touch_info)))
    }
}

#[derive(Clone)]
pub struct TouchPositionFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + Clone> TouchFn
    for TouchPositionFunc<F>
{
    fn call(&self, cx: &mut Context, touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx, touch_info.pt, touch_info.button)))
    }
}

#[derive(Clone)]
pub struct TouchAdapter<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context) -> A + Clone> TouchFn for TouchAdapter<F> {
    fn call(&self, cx: &mut Context, _touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx)))
    }
}

#[derive(Clone)]
pub struct TouchActionAdapter<A> {
    pub action: A,
}

impl<A: Clone + 'static> TouchFn for TouchActionAdapter<A> {
    fn call(&self, _cx: &mut Context, _touch_info: TouchInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new(self.action.clone()))
    }
}

/// Struct for the `touch` gesture.
#[derive(Clone)]
pub struct Touch<V: View, F> {
    /// Child view tree.
    child: V,

    /// Called when a touch occurs.
    func: F,
}

impl<V, F> Touch<V, F>
where
    V: View,
    F: TouchFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Touch<V, F>
where
    V: View,
    F: TouchFn + 'static,
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
                    self.func.call(
                        cx,
                        TouchInfo {
                            pt: *position,
                            button: cx.mouse_button,
                            state: TouchState::Begin,
                        },
                        actions,
                    )
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    self.func.call(
                        cx,
                        TouchInfo {
                            pt: *position,
                            button: cx.mouse_button,
                            state: TouchState::End,
                        },
                        actions,
                    )
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

impl<V, F> private::Sealed for Touch<V, F> where V: View {}
