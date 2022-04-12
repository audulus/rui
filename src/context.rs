use crate::*;
use euclid::*;
use std::any::Any;
use std::collections::HashMap;
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

pub type StateMap = HashMap<ViewID, Box<dyn Any>>;

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
    pub(crate) state_map: StateMap,

    /// Has the state changed?
    pub(crate) dirty: bool,

    /// Are we currently setting the dirty bit?
    pub(crate) enable_dirty: bool,
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
            dirty: false,
            enable_dirty: true
        }
    }

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub fn get<S>(&self, id: State<S>) -> &S
    where
        S: 'static,
    {
        self.state_map[&id.id].downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: State<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();
        self.state_map
            .get_mut(&id.id)
            .unwrap()
            .downcast_mut::<S>()
            .unwrap()
    }
}

impl<S> ops::Index<State<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: State<S>) -> &Self::Output {
        self.state_map[&index.id].downcast_ref::<S>().unwrap()
    }
}

impl<S> ops::IndexMut<State<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: State<S>) -> &mut Self::Output {
        self.set_dirty();
        self.state_map
            .get_mut(&index.id)
            .unwrap()
            .downcast_mut::<S>()
            .unwrap()
    }
}

