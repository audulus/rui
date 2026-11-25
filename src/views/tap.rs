use crate::*;
use std::any::Any;

pub struct TapInfo {
    pub pt: LocalPoint,
    pub button: Option<MouseButton>,
    pub state: TouchState,
}

pub trait TapFn: Clone {
    fn call(&self, cx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>);
}

#[derive(Clone)]
pub struct TapFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, TapInfo) -> A + Clone + 'static> TapFn for TapFunc<F> {
    fn call(&self, cx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx, tap_info)))
    }
}

#[derive(Clone)]
pub struct TapPositionFunc<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context, LocalPoint, Option<MouseButton>) -> A + Clone + 'static> TapFn
    for TapPositionFunc<F>
{
    fn call(&self, cx: &mut Context, tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx, tap_info.pt, tap_info.button)))
    }
}

#[derive(Clone)]
pub struct TapAdapter<F> {
    pub f: F,
}

impl<A: 'static, F: Fn(&mut Context) -> A + Clone + 'static> TapFn for TapAdapter<F> {
    fn call(&self, cx: &mut Context, _tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new((self.f)(cx)))
    }
}

#[derive(Clone)]
pub struct TapActionAdapter<A> {
    pub action: A,
}

impl<A: Clone + 'static> TapFn for TapActionAdapter<A> {
    fn call(&self, _cx: &mut Context, _tap_info: TapInfo, actions: &mut Vec<Box<dyn Any>>) {
        actions.push(Box::new(self.action.clone()))
    }
}

/// Struct for the `tap` gesture.
#[derive(Clone)]
pub struct Tap<V, F> {
    /// Child view tree.
    child: V,

    /// Called when a tap occurs.
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> DynView for Tap<V, F>
where
    V: View,
    F: TapFn + 'static,
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
                }
            }
            Event::TouchEnd { id, position } if cx.touches[*id] == vid => {
                cx.touches[*id] = ViewId::default();
                self.func.call(
                    cx,
                    TapInfo {
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

impl<V, F> private::Sealed for Tap<V, F> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tap_fires_on_touch_end() {
        let mut cx = Context::new();

        let ui = state(
            || false,
            |tapped, _| {
                rectangle().tap(move |cx| {
                    *tapped.get_mut(cx) = true;
                })
            },
        );

        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );

        let s = StateHandle::<bool>::new(vid);
        assert!(!*s.get(&cx));

        let events = [
            Event::TouchBegin {
                id: 0,
                position: [50.0, 50.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [50.0, 50.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, vid, &mut cx, &mut actions);
        }

        assert!(*s.get(&cx));
    }

    #[test]
    fn test_tap_outside_does_not_fire() {
        let mut cx = Context::new();

        let ui = state(
            || false,
            |tapped, _| {
                rectangle().size([50.0, 50.0]).tap(move |cx| {
                    *tapped.get_mut(cx) = true;
                })
            },
        );

        let sz = [100.0, 100.0].into();
        let vid = cx.root_id();
        ui.layout(
            vid,
            &mut LayoutArgs {
                sz,
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );

        let s = StateHandle::<bool>::new(vid);

        // Touch outside the 50x50 rectangle
        let events = [
            Event::TouchBegin {
                id: 0,
                position: [80.0, 80.0].into(),
            },
            Event::TouchEnd {
                id: 0,
                position: [80.0, 80.0].into(),
            },
        ];

        let mut actions = vec![];
        for event in &events {
            ui.process(event, vid, &mut cx, &mut actions);
        }

        assert!(!*s.get(&cx));
    }
}
