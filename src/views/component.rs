use crate::*;
use std::any::{Any, TypeId};

/// A reusable piece of UI defined by a struct and a `body`.
///
/// This is rui's analogue of a SwiftUI `View`: the struct's fields are the
/// component's inputs ("props"), and `body` describes the UI built from them.
/// Any `Component` is automatically a [`View`], so it can be used directly in
/// stacks and with modifiers.
///
/// ```no_run
/// use rui::*;
///
/// #[derive(Clone)]
/// struct Greeting {
///     name: String,
/// }
///
/// impl Component for Greeting {
///     fn body(&self) -> impl View {
///         text(&format!("Hello, {}!", self.name)).padding(Auto)
///     }
/// }
///
/// vstack((Greeting { name: "rui".into() }, Greeting { name: "world".into() })).run();
/// ```
///
/// `body` takes no `Context`, so a component's structure depends only on its
/// fields. To give a component state of its own, implement
/// [`StatefulComponent`] instead; to read state owned elsewhere, take a
/// [`Binding`] as a field and use [`with_ref`] inside `body`.
pub trait Component: Clone + 'static {
    fn body(&self) -> impl View;
}

/// A component that declares its own state.
///
/// `State` is stored by rui at the component's position in the view tree,
/// like SwiftUI's `@State`: it is created with [`init`](Self::init) the first
/// time the component appears, and kept across re-renders even when the
/// parent passes new props.
///
/// `body` receives the current state value to read, and a [`StateHandle`]
/// to change it from event handlers:
///
/// ```no_run
/// use rui::*;
///
/// #[derive(Clone)]
/// struct Counter {
///     step: i32,
/// }
///
/// impl StatefulComponent for Counter {
///     type State = i32;
///
///     fn init(&self) -> i32 {
///         0
///     }
///
///     fn body(self, count: i32, set: StateHandle<i32>) -> impl View {
///         button(format!("count: {count}"), move |cx| cx[set] += self.step)
///     }
/// }
///
/// vstack((Counter { step: 1 }, Counter { step: 10 })).run();
/// ```
///
/// `body` takes `self` and the state by value, so the returned view borrows
/// nothing and impls can write a plain `-> impl View`. rui already asks for
/// state to be cheap to clone; use `Rc` for large state.
///
/// The [`component`] attribute macro writes this impl for you from
/// `#[state]` fields.
///
/// Every `StatefulComponent` is also a [`Component`] (and so a [`View`]).
/// Like [`state`], a stateful component is never flexible in a stack.
pub trait StatefulComponent: Clone + 'static {
    /// The component's state.
    type State: Clone + 'static;

    /// Creates the initial state, which may depend on props. Called once,
    /// when the component first appears at its position.
    fn init(&self) -> Self::State;

    fn body(self, state: Self::State, handle: StateHandle<Self::State>) -> impl View;
}

impl<C: StatefulComponent> Component for C {
    fn body(&self) -> impl View {
        let this = self.clone();
        let init = self.clone();
        state(
            move || init.init(),
            move |s, cx| StatefulComponent::body(this.clone(), cx[s].clone(), s),
        )
    }
}

impl<C: Component> private::Sealed for C {}

// The body is a transparent stand-in for the component: it is laid out,
// drawn and hit-tested at the component's own path, so a component adds
// no level to the ViewId path.
impl<C: Component> DynView for C {
    fn access(
        &self,
        path: &mut IdPath,
        cx: &mut Context,
        nodes: &mut Vec<(accesskit::NodeId, accesskit::Node)>,
    ) -> Option<accesskit::NodeId> {
        self.body().access(path, cx, nodes)
    }

    fn commands(&self, path: &mut IdPath, cx: &mut Context, cmds: &mut Vec<CommandInfo>) {
        self.body().commands(path, cx, cmds)
    }

    fn dirty(&self, path: &mut IdPath, xform: LocalToWorld, cx: &mut Context) {
        self.body().dirty(path, xform, cx)
    }

    fn draw(&self, path: &mut IdPath, args: &mut DrawArgs) {
        self.body().draw(path, args)
    }

    fn gc(&self, path: &mut IdPath, cx: &mut Context, map: &mut Vec<ViewId>) {
        self.body().gc(path, cx, map)
    }

    fn hittest(&self, path: &mut IdPath, pt: LocalPoint, cx: &mut Context) -> Option<ViewId> {
        self.body().hittest(path, pt, cx)
    }

    fn is_flexible(&self) -> bool {
        self.body().is_flexible()
    }

    fn layout(&self, path: &mut IdPath, args: &mut LayoutArgs) -> LocalSize {
        self.body().layout(path, args)
    }

    fn process(
        &self,
        event: &Event,
        path: &mut IdPath,
        cx: &mut Context,
        actions: &mut Vec<Box<dyn Any>>,
    ) {
        self.body().process(event, path, cx, actions)
    }

    fn tid(&self) -> TypeId {
        TypeId::of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_harness::TestHarness;

    #[derive(Clone)]
    struct Box2 {
        w: f32,
        h: f32,
    }

    impl Component for Box2 {
        fn body(&self) -> impl View {
            rectangle().size([self.w, self.h])
        }
    }

    #[derive(Clone)]
    struct Spacey;

    impl Component for Spacey {
        fn body(&self) -> impl View {
            spacer()
        }
    }

    #[derive(Clone)]
    struct Counter {
        step: i32,
    }

    impl Component for Counter {
        fn body(&self) -> impl View {
            let step = self.step;
            state(
                || 0i32,
                move |count, _| {
                    rectangle()
                        .size([100.0, 50.0])
                        .tap(move |cx| cx[count] += step)
                },
            )
        }
    }

    /// Edits state owned by its parent.
    #[derive(Clone)]
    struct Incrementer<B> {
        value: B,
    }

    impl<B: Binding<i32>> Component for Incrementer<B> {
        fn body(&self) -> impl View {
            let value = self.value;
            rectangle()
                .size([100.0, 50.0])
                .tap(move |cx| value.with_mut(cx, |v| *v += 1))
        }
    }

    #[test]
    fn component_edits_parent_state_through_binding() {
        let ui = state(|| 0i32, |s, _| Incrementer { value: s });
        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        h.tap([10.0, 10.0]);
        h.tap([10.0, 10.0]);
        h.tap([10.0, 10.0]);
        assert_eq!(h.cx[s], 3);
    }

    #[derive(Clone)]
    struct Clicker {
        step: i32,
    }

    impl StatefulComponent for Clicker {
        type State = i32;

        fn init(&self) -> i32 {
            0
        }

        fn body(self, _count: i32, set: StateHandle<i32>) -> impl View {
            rectangle()
                .size([100.0, 50.0])
                .tap(move |cx| cx[set] += self.step)
        }
    }

    #[derive(Clone)]
    struct StartsAt {
        start: i32,
    }

    impl StatefulComponent for StartsAt {
        type State = i32;

        fn init(&self) -> i32 {
            self.start
        }

        fn body(self, _count: i32, set: StateHandle<i32>) -> impl View {
            rectangle()
                .size([100.0, 50.0])
                .tap(move |cx| cx[set] += 1)
        }
    }

    #[test]
    fn stateful_component_keeps_its_own_state() {
        let ui = Clicker { step: 3 };
        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        h.tap([10.0, 10.0]);
        h.tap([10.0, 10.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], 6);
    }

    #[test]
    fn stateful_component_init_uses_props() {
        let ui = StartsAt { start: 100 };
        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[s], 100);
        h.tap([10.0, 10.0]);
        assert_eq!(h.cx[s], 101);
    }

    #[test]
    fn stateful_siblings_get_separate_state() {
        let ui = vstack((Clicker { step: 1 }, Clicker { step: 2 }));
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // y-up coordinates: the first child is on top (y 50..100).
        h.tap([10.0, 75.0]);
        h.tap([10.0, 25.0]);
        let first = StateHandle::<i32>::new(h.cx.view_id(&vec![0, 0]));
        let second = StateHandle::<i32>::new(h.cx.view_id(&vec![0, 1]));
        assert_eq!(h.cx[first], 1);
        assert_eq!(h.cx[second], 2);
    }

    #[test]
    fn stateful_component_state_survives_prop_changes() {
        // The parent re-renders with a new `step` each tap; the child's
        // count must carry over rather than being re-initialized.
        let ui = state(
            || 1i32,
            |step, cx| {
                let s = cx[step];
                vstack((
                    rectangle()
                        .size([100.0, 50.0])
                        .tap(move |cx| cx[step] += 1),
                    Clicker { step: s },
                ))
            },
        );
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        h.tap([10.0, 25.0]); // child: +1
        h.tap([10.0, 75.0]); // parent: step = 2
        h.tap([10.0, 25.0]); // child: +2
        let child = StateHandle::<i32>::new(h.cx.view_id(&vec![0, 0, 1]));
        assert_eq!(h.cx[child], 3);
    }

    // --- #[component] macro ---

    #[component]
    #[derive(Clone)]
    struct Tally {
        step: i32,
        #[state]
        count: i32,
        #[state = self.step * 100]
        start: i32,
        #[state = false]
        on: bool,
    }

    impl Tally {
        fn body(self, state: TallyState, set: TallySet) -> impl View {
            let _ = state.start;
            vstack((
                rectangle()
                    .size([100.0, 50.0])
                    .tap(move |cx| cx[set.count] += self.step),
                toggle(set.on),
            ))
        }
    }

    fn tally_state(h: &mut TestHarness<'_, impl View>, path: &[u64]) -> TallyState {
        let s = StateHandle::<TallyState>::new(h.cx.view_id(&path.to_vec()));
        h.cx[s].clone()
    }

    #[test]
    fn macro_component_built_from_props_only() {
        fn takes_view(_: impl View) {}
        takes_view(Tally { step: 1 });
    }

    #[test]
    fn macro_component_initial_state() {
        let ui = Tally { step: 3 };
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        let s = tally_state(&mut h, &[0]);
        assert_eq!(s.count, 0); // #[state] => Default
        assert_eq!(s.start, 300); // #[state = expr] can read props
        assert!(!s.on);
    }

    #[test]
    fn macro_component_writes_through_set() {
        let ui = Tally { step: 7 };
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // y-up: the tap rectangle is on top, the toggle below it.
        h.tap([10.0, 75.0]);
        h.tap([10.0, 75.0]);
        assert_eq!(tally_state(&mut h, &[0]).count, 14);
    }

    #[test]
    fn macro_component_passes_binding_to_child() {
        let ui = Tally { step: 1 };
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // Tap the toggle (35x20, at the bottom-left), which edits `on`
        // through `set.on`.
        h.tap([10.0, 10.0]);
        assert!(tally_state(&mut h, &[0]).on);
    }

    #[test]
    fn macro_component_state_survives_prop_changes() {
        let ui = state(
            || 1i32,
            |step, cx| {
                let s = cx[step];
                hstack((
                    rectangle()
                        .size([50.0, 50.0])
                        .tap(move |cx| cx[step] += 1),
                    Tally { step: s },
                ))
            },
        );
        let mut h = TestHarness::new(&ui, [150.0, 100.0]);
        let tally_path = [0, 0, 1];
        let before = tally_state(&mut h, &tally_path);
        assert_eq!(before.start, 100);

        h.tap([10.0, 50.0]); // parent: step = 2
        let after = tally_state(&mut h, &tally_path);
        // init ran once; new props don't reset state.
        assert_eq!(after.start, 100);
        let step = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        assert_eq!(h.cx[step], 2);
    }

    #[test]
    fn context_indexes_lens_bindings() {
        let ui = state(|| 0i32, |_, _| EmptyView {});
        let mut h = TestHarness::new(&ui, [10.0, 10.0]);
        let s = StateHandle::<TallyState>::new(h.cx.view_id(&vec![9]));
        h.cx.init_state(s.id, &|| TallyState {
            count: 1,
            start: 2,
            on: false,
        });
        let b = Map::new(s, __TallyCountLens);
        h.cx[b] += 41;
        assert_eq!(h.cx[b], 42);
        assert_eq!(h.cx[s].count, 42);
    }

    #[test]
    fn component_is_a_view() {
        fn takes_view(_: impl View) {}
        takes_view(Box2 { w: 1.0, h: 2.0 });
        takes_view(Box2 { w: 1.0, h: 2.0 }.padding(Auto));
    }

    #[test]
    fn component_layout_matches_body() {
        let mut cx = Context::new();
        let mut path = vec![0];
        let sz = Box2 { w: 30.0, h: 20.0 }.layout(
            &mut path,
            &mut LayoutArgs {
                sz: [100.0, 100.0].into(),
                cx: &mut cx,
                text_bounds: &mut |_, _, _| LocalRect::zero(),
            },
        );
        assert_eq!(sz, LocalSize::new(30.0, 20.0));
    }

    #[test]
    fn component_flexibility_comes_from_body() {
        assert!(Spacey.is_flexible());
        assert!(!Box2 { w: 1.0, h: 1.0 }.is_flexible());
    }

    #[test]
    fn component_distinct_type_ids() {
        let a = any_view(Box2 { w: 1.0, h: 1.0 });
        let b = any_view(Spacey);
        assert_ne!(a.tid(), b.tid());
        assert_eq!(a.tid(), TypeId::of::<Box2>());
    }

    #[test]
    fn component_with_state_handles_taps() {
        let ui = Counter { step: 5 };
        let mut h = TestHarness::new(&ui, [100.0, 50.0]);
        // The component adds no path level, so its body's state lives at [0].
        let s = StateHandle::<i32>::new(h.cx.view_id(&vec![0]));
        h.tap([10.0, 10.0]);
        h.tap([10.0, 10.0]);
        assert_eq!(h.cx[s], 10);
    }

    #[test]
    fn components_in_a_stack_get_separate_state() {
        let ui = vstack((Counter { step: 1 }, Counter { step: 2 }));
        let mut h = TestHarness::new(&ui, [100.0, 100.0]);
        // y-up coordinates: the first child is on top (y 50..100).
        h.tap([10.0, 75.0]);
        h.tap([10.0, 25.0]);
        h.tap([10.0, 25.0]);
        let first = StateHandle::<i32>::new(h.cx.view_id(&vec![0, 0]));
        let second = StateHandle::<i32>::new(h.cx.view_id(&vec![0, 1]));
        assert_eq!(h.cx[first], 1);
        assert_eq!(h.cx[second], 4);
    }
}
