use crate::*;
use std::any::Any;
use std::marker::PhantomData;

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum GestureState {
    Began,
    Changed,
    Ended,
}

pub trait DragFn {
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

pub struct DragFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalOffset, GestureState, Option<MouseButton>) -> A> DragFn
    for DragFunc<F>
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

pub struct DragFuncP<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint, GestureState, Option<MouseButton>) -> A> DragFn
    for DragFuncP<F>
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

pub struct DragFuncS<F, B, T> {
    pub f: F,
    pub b: B,
    pub phantom: PhantomData<T>,
}

impl<
        F: Fn(&mut T, LocalOffset, GestureState, Option<MouseButton>) -> A + 'static,
        B: Binding<T>,
        A: 'static,
        T: 'static,
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
        actions.push(Box::new((self.f)(
            self.b.get_mut(cx),
            delta,
            state,
            button,
        )))
    }
}

/// Struct for the `drag` and `drag_p` gestures.
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

impl<V, F> View for Drag<V, F>
where
    V: View,
    F: DragFn + 'static,
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
                if cx.touches[*id].is_default() && self.hittest(path, *position, cx).is_some() {
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

impl<V, F> private::Sealed for Drag<V, F> {}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_drag() {
        let mut cx = Context::new();

        let ui = state(
            || vec![],
            |states, _| rectangle().drag(move |cx, _delta, state, _| cx[states].push(state)),
        );
        let sz = [100.0, 100.0].into();
        let mut path = vec![0];

        let rect_sz = ui.layout(
            &mut path,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::new(LocalPoint::zero(), [90.0, 90.0].into()),
            },
        );
        assert_eq!(path.len(), 1);

        assert_eq!(rect_sz, sz);
        let s = StateHandle::<Vec<GestureState>>::new(cx.view_id(&path));
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
            ui.process(event, &mut path, &mut cx, &mut actions);
        }
        assert_eq!(path.len(), 1);

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
