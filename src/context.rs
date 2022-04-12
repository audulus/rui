use crate::*;
use euclid::*;
use std::collections::HashMap;
use std::any::Any;
use std::ops;

pub type LocalSpace = vger::defs::LocalSpace;
pub type WorldSpace = vger::defs::WorldSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;
pub type WorldPoint = Point2D<f32, WorldSpace>;

use tao::window::Window;

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Layout information for all views.
    pub(crate) layout: HashMap<ViewID, LayoutBox>,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewID; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [LocalPoint; 16],

    /// The root view ID. This should be randomized for security reasons.
    pub(crate) root_id: ViewID,

    /// The view that has the keybord focus.
    pub(crate) focused_id: Option<ViewID>,

    /// The tao window
    pub(crate) window: Option<Window>,

    /// The current title of the window
    pub(crate) window_title: String,

    /// Attempt to not use interior mutability.
    pub(crate) state_map: HashMap<ViewID, Box<dyn Any>>,
}

impl Context {
    pub fn new(window: Option<Window>) -> Self {
        Self {
            layout: HashMap::new(),
            touches: [ViewID::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            root_id: ViewID { id: 1 },
            focused_id: None,
            window,
            window_title: "rui".into(),
            state_map: HashMap::new(),
        }
    }

    pub fn get<S>(&self, id: State<S>) -> &S where S: 'static {
        self.state_map[&id.id].downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: State<S>) -> &mut S where S: 'static {
        set_state_dirty();
        self.state_map.get_mut(&id.id).unwrap().downcast_mut::<S>().unwrap()
    }
}

impl<S> ops::Index<State<S>> for Context where S: 'static {
    type Output = S;

    fn index(&self, index: State<S>) -> &Self::Output {
        self.state_map[&index.id].downcast_ref::<S>().unwrap()
    }
}

impl<S> ops::IndexMut<State<S>> for Context where S: 'static {
    fn index_mut(&mut self, index: State<S>) -> &mut Self::Output {
        set_state_dirty();
        self.state_map.get_mut(&index.id).unwrap().downcast_mut::<S>().unwrap()
    }
}

pub trait Lens<T, U>: Clone + Copy + 'static {
    fn focus<'a>(&self, data: &'a T) -> &'a U;
    fn focus_mut<'a>(&self, data: &'a mut T) -> &'a mut U;
}

/// Reads or writes a value owned by a source-of-truth.
pub trait Binding2<S>: Clone + Copy + 'static {
    fn get2<'a>(&self, cx: &'a mut Context) -> &'a S;
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S;
}

#[derive(Clone)]
pub struct Map2<B, L, T> {
    binding: B,
    lens: L,
    phantom: std::marker::PhantomData<T>,
}

impl<B, L, T> Copy for Map2<B, L, T> where B: Copy, L: Copy, T: Clone {}

impl<S, B, L, T> Binding2<S> for Map2<B, L, T> where B: Binding2<T>, L: Lens<T, S>, S: Clone + 'static, T: Clone + 'static {
    fn get2<'a>(&self, cx: &'a mut Context) -> &'a S {
        self.lens.focus(self.binding.get2(cx))
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        self.lens.focus_mut(self.binding.get_mut(cx))
    }
}

#[derive(Clone)]
pub struct Map3<B, F, FM, T> {
    binding: B,
    focus: F,
    focus_mut: FM,
    phantom: std::marker::PhantomData<T>,
}

impl<B, F, FM, T> Copy for Map3<B, F, FM, T> where B: Copy, F: Copy, FM: Copy, T: Clone {}

impl<S, B, F, FM, T> Binding2<S> for Map3<B, F, FM, T> 
where
    B: Binding2<T>,
    F: Fn(&T) -> &S + Copy + 'static,
    FM: Fn(&mut T) -> &mut S + Copy + 'static,
    S: Clone + 'static,
    T: Clone + 'static {
    fn get2<'a>(&self, cx: &'a mut Context) -> &'a S {
        (self.focus)(self.binding.get2(cx))
    }
    fn get_mut<'a>(&self, cx: &'a mut Context) -> &'a mut S {
        (self.focus_mut)(self.binding.get_mut(cx))
    }
}

#[macro_export]
macro_rules! bind2 {
    ( $state:expr, $field:ident, $t:ty ) => {{
        let s = $state;
        Map3::<_,_,_,$t> {
            binding: s,
            focus: |x: & $t| x.$field,
            focus_mut: |x: &mut $t| x.$field,
            phantom: Default::default(),
        }
    }};
}

#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone)]
    struct MyState {
        x: i32
    }

    #[derive(Clone, Copy)]
    struct MyLens {}
    impl Lens<MyState, i32> for MyLens {
        fn focus<'a>(&self, data: &'a MyState) -> &'a i32 {
            &data.x
        }
        fn focus_mut<'a>(&self, data: &'a mut MyState) -> &'a mut i32 {
            &mut data.x
        }
    }

    #[test]
    fn test_lens() {

        let mut s = MyState{ x: 0 };
        *MyLens{}.focus_mut(&mut s) = 42;
        assert_eq!(*MyLens{}.focus(&s), 42);

    }

    #[test]
    fn test_bind2() {

        let mut cx = Context::new(None);
        let id = ViewID::default();
        cx.state_map.entry(id).or_insert_with(|| Box::new(MyState{ x: 0}));
        let s = State::new(id, &|| MyState{ x: 0 });

        let b = Map2 {
            binding: s,
            lens: MyLens{},
            phantom: std::marker::PhantomData::<MyState>{}
        };

        *b.get_mut(&mut cx) = 42;
    }
}