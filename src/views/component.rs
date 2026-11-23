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
/// fields. To read or create state, use [`state`], [`with_ref`] or
/// [`with_cx`] inside `body`.
pub trait Component: Clone + 'static {
    fn body(&self) -> impl View;
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
