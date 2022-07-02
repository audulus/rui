use crate::*;
use std::any::Any;

/// Struct for the `tap` gesture.
pub struct Tap<V, F> {
    child: V,
    func: F,
}

impl<V, F, A> Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) -> A + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F, A> View for Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) -> A + 'static,
    A: 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(vid, *position, cx, vger).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    actions.push(Box::new((self.func)(cx)))
                }
            }
            _ => (),
        }
    }

    fn draw(&self, id: ViewId, cx: &mut Context, vger: &mut Vger) {
        self.child.draw(id.child(&0), cx, vger)
    }

    fn layout(&self, id: ViewId, sz: LocalSize, cx: &mut Context, vger: &mut Vger) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger)
    }

    fn hittest(
        &self,
        id: ViewId,
        pt: LocalPoint,
        cx: &mut Context,
        vger: &mut Vger,
    ) -> Option<ViewId> {
        self.child.hittest(id.child(&0), pt, cx, vger)
    }

    fn commands(&self, id: ViewId, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.child.commands(id.child(&0), cx, cmds)
    }

    fn gc(&self, id: ViewId, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.child.gc(id.child(&0), cx, map)
    }

    fn access(
        &self,
        id: ViewId,
        cx: &mut Context,
        nodes: &mut Vec<accesskit::Node>,
    ) -> Option<accesskit::NodeId> {
        self.child.access(id.child(&0), cx, nodes)
    }
}

impl<V, F> private::Sealed for Tap<V, F> {}

pub struct Tap2<'a, V, F, Data>
where
    F: Fn(&mut Data) + 'a,
{
    child: V,
    func: F,
    // XXX: seems a bit sketchy to use &'a i32 to satisfy the compiler.
    phantom: std::marker::PhantomData<fn() -> (Data, &'a i32)>,
}

impl<'a, V, F, Data> Tap2<'a, V, F, Data>
where
    V: View2<Data>,
    Data: Sized,
    F: Fn(&mut Data) + 'a,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            phantom: Default::default(),
        }
    }
}

impl<'a, V, F, Data> View2<Data> for Tap2<'a, V, F, Data>
where
    V: View2<Data>,
    Data: 'static,
    F: Fn(&mut Data) + 'a,
{
    type State = V::State;

    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        data: &mut Data,
    ) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self
                    .hittest(vid, *position, cx, vger, state, data)
                    .is_some()
                {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    (self.func)(data);
                }
            }
            _ => (),
        }
    }

    fn draw(
        &self,
        id: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        data: &Data,
    ) {
        self.child.draw(id.child(&0), cx, vger, state, data)
    }

    fn layout(
        &self,
        id: ViewId,
        sz: LocalSize,
        cx: &mut Context,
        vger: &mut Vger,
        state: &mut Self::State,
        data: &Data,
    ) -> LocalSize {
        self.child.layout(id.child(&0), sz, cx, vger, state, data)
    }
}
