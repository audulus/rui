use crate::*;
use std::any::Any;

#[derive(Clone, PartialEq)]
pub enum TouchState {
    Begin,
    End,
}

#[derive(Clone)]
pub struct TouchInfo {
    /// The position of the touch in local space of the view that received the touch.
    pub pt: LocalPoint,

    /// The mouse button that was used for the touch if a mouse was used.
    pub button: Option<MouseButton>,

    /// The state of the touch. IE: Begin or End.
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
        vid: ViewId,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(vid, *position, cx).is_some() {
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
            Event::TouchEnd { id, position } if cx.touches[*id] == vid => {
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
            _ => (),
        }
    }

    fn draw(&self, vid: ViewId, args: &mut DrawArgs) {
        let cid = args.cx.child(vid, 0);
        self.child.draw(cid, args);
    }

    fn layout(&self, vid: ViewId, args: &mut LayoutArgs) -> LocalSize {
        let cid = args.cx.child(vid, 0);

        self.child.layout(cid, args)
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

impl<V, F> private::Sealed for Touch<V, F> where V: View {}
