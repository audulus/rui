use crate::*;

/// Struct for the `tap` gesture.
pub struct Tap<V, F> {
    child: V,
    func: F,
}

impl<V, F> Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self { child: v, func: f }
    }
}

impl<V, F> View for Tap<V, F>
where
    V: View,
    F: Fn(&mut Context) + 'static,
{
    fn print(&self, id: ViewId, cx: &mut Context) {
        println!("Tap {{");
        (self.child).print(id.child(&0), cx);
        println!("}}");
    }

    fn process(&self, event: &Event, vid: ViewId, cx: &mut Context, vger: &mut Vger) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self.hittest(vid, *position, cx, vger).is_some() {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    (self.func)(cx);
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

pub struct Tap2<V, F, Data> {
    child: V,
    func: F,
    data: std::marker::PhantomData<Data>,
}

impl<V, F, Data> Tap2<V, F, Data>
where
    V: View2<Data>,
    Data: Sized,
    F: Fn(&mut Data) + 'static,
{
    pub fn new(v: V, f: F) -> Self {
        Self {
            child: v,
            func: f,
            data: Default::default(),
        }
    }
}

impl<V, F, Data> View2<Data> for Tap2<V, F, Data>
where
    V: View2<Data>,
    Data: 'static,
    F: Fn(&mut Data) + 'static,
{
    fn process(
        &self,
        event: &Event,
        vid: ViewId,
        cx: &mut Context,
        vger: &mut Vger,
        state0: &mut StateStorage,
        state1: &mut StateStorage,
        state2: &mut StateStorage,
        state_level: usize,
        data: State<Data>,
    ) {
        match &event {
            Event::TouchBegin { id, position } => {
                if self
                    .hittest(
                        vid,
                        *position,
                        cx,
                        vger,
                        state0,
                        state1,
                        state2,
                        state_level,
                    )
                    .is_some()
                {
                    cx.touches[*id] = vid;
                }
            }
            Event::TouchEnd { id, position: _ } => {
                if cx.touches[*id] == vid {
                    cx.touches[*id] = ViewId::default();
                    (self.func)(cx.get_mut(data));
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
        state0: &mut StateStorage,
        state1: &mut StateStorage,
        state2: &mut StateStorage,
        state_level: usize,
    ) {
        self.child
            .draw(id.child(&0), cx, vger, state0, state1, state2, state_level)
    }

    fn layout(
        &self,
        id: ViewId,
        sz: LocalSize,
        cx: &mut Context,
        vger: &mut Vger,
        state0: &mut StateStorage,
        state1: &mut StateStorage,
        state2: &mut StateStorage,
        state_level: usize,
    ) -> LocalSize {
        self.child.layout(
            id.child(&0),
            sz,
            cx,
            vger,
            state0,
            state1,
            state2,
            state_level,
        )
    }
}
