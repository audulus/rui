use crate::*;
use std::any::Any;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum GestureState {
    Began,
    Changed,
    Ended,
}

pub trait DragFn: Clone {
    fn call(
        &self,
        cx: &mut Context,
        pt: LocalPoint,
        delta: LocalOffset,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    );
}

#[derive(Clone)]
pub struct DragFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalOffset, GestureState, Option<MouseButton>) -> A + Clone>
    DragFn for DragFunc<F>
{
    fn call(
        &self,
        cx: &mut Context,
        _pt: LocalPoint,
        delta: LocalOffset,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(cx, delta, state, button)))
    }
}

#[derive(Clone)]
pub struct DragFuncP<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint, GestureState, Option<MouseButton>) -> A + Clone>
    DragFn for DragFuncP<F>
{
    fn call(
        &self,
        cx: &mut Context,
        pt: LocalPoint,
        _delta: LocalOffset,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(cx, pt, state, button)))
    }
}

#[derive(Clone)]
pub struct DragFuncS<F, B, T> {
    pub f: F,
    pub b: B,
    pub phantom: PhantomData<T>,
}

impl<
    F: Fn(&mut T, LocalOffset, GestureState, Option<MouseButton>) -> A + Clone + 'static,
    B: Binding<T>,
    A: 'static,
    T: Clone + 'static,
> DragFn for DragFuncS<F, B, T>
{
    fn call(
        &self,
        cx: &mut Context,
        _pt: LocalPoint,
        delta: LocalOffset,
        state: GestureState,
        button: Option<MouseButton>,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        actions.push(Box::new((self.f)(self.b.get_mut(cx), delta, state, button)))
    }
}

/// Struct for the `drag` and `drag_p` gestures.
#[derive(Clone)]
pub struct Drag<V, F> {
    child: V,
    func: F,
    grab: bool,
}

impl<V, F> Drag<V, F>
where
    V: View,
    F: DragFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            grab: false,
        }
    }

    pub fn grab_cursor(self) -> Self {
        Self {
            child: self.child,
            func: self.func,
            grab: true,
        }
    }
}

impl<V, F> DynView for Drag<V, F>
where
    V: View,
    F: DragFn + 'static,
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
                if cx.touches[*id].is_default() && self.hittest(vid, *position, cx).is_some() {
                    cx.touches[*id] = vid;
                    cx.starts[*id] = *position;
                    cx.previous_position[*id] = *position;
                    cx.grab_cursor = self.grab;

                    self.func.call(
                        cx,
                        *position,
                        LocalOffset::zero(),
                        GestureState::Began,
                        cx.mouse_button,
                        actions,
                    );
                }
            }
            Event::TouchMove {
                id,
                position,
                delta,
            } => {
                if cx.touches[*id] == vid {
                    self.func.call(
                        cx,
                        *position,
                        *delta,
                        GestureState::Changed,
                        cx.mouse_button,
                        actions,
                    );
                    cx.previous_position[*id] = *position;
                }
            }
            Event::TouchEnd { id, position } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    cx.grab_cursor = false;

                    self.func.call(
                        cx,
                        *position,
                        LocalOffset::zero(),
                        GestureState::Ended,
                        cx.mouse_button,
                        actions,
                    );
                }
            }
            _ => {
                let cid = cx.child(vid, 0);
                self.child.process(event, cid, cx, actions);
            }
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

impl<V, F> private::Sealed for Drag<V, F> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_drag() {
        let mut cx = Context::new();

        let ui = state(std::vec::Vec::new, |states, _| {
            rectangle().drag(move |cx, _delta, state, _| cx[states].push(state))
        });
        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();

        let rect_sz = ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::new(LocalPoint::zero(), [90.0, 90.0].into()),
            },
        );

        assert_eq!(rect_sz, sz);
        let s = StateHandle::<Vec<GestureState>>::new(vid);
        assert_eq!(cx[s], vec![]);

        let events = [
            Event::TouchBegin {
                id: 0,
                position: [50.0, 50.0].into(),
            },
            Event::TouchMove {
                id: 0,
                position: [60.0, 50.0].into(),
                delta: [10.0, 0.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [60.0, 50.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, vid, &mut cx, &mut actions);
        }

        assert_eq!(
            cx[s],
            vec![
                GestureState::Began,
                GestureState::Changed,
                GestureState::Ended
            ]
        );
    }
}
