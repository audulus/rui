use crate::*;
use euclid::*;
use std::any::Any;
use std::collections::HashMap;
use std::ops;
use tao::event::MouseButton;

pub type LocalSpace = vger::defs::LocalSpace;
pub type WorldSpace = vger::defs::WorldSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;
pub type WorldPoint = Point2D<f32, WorldSpace>;
pub type LocalToWorld = Transform2D<f32, LocalSpace, WorldSpace>;
pub type WorldToLocal = Transform2D<f32, WorldSpace, LocalSpace>;

use tao::window::Window;

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub(crate) struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

pub(crate) struct StateHolder {
    pub state: Box<dyn Any>,
    pub dirty: bool,
}

pub(crate) type StateMap = HashMap<ViewId, StateHolder>;

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Layout information for all views.
    pub(crate) layout: HashMap<ViewId, LayoutBox>,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub(crate) touches: [ViewId; 16],

    /// Points at which touches (or click-drags) started.
    pub(crate) starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub(crate) previous_position: [LocalPoint; 16],

    /// Pressed mouse buton.
    pub(crate) mouse_button: Option<MouseButton>,

    /// Keyboard modifiers state.
    pub(crate) key_mods: ModifiersState,

    /// The root view ID. This should be randomized for security reasons.
    pub(crate) root_id: ViewId,

    /// The view that has the keybord focus.
    pub(crate) focused_id: Option<ViewId>,

    /// The tao window
    pub(crate) window: Option<Window>,

    /// The current title of the window
    pub(crate) window_title: String,

    /// User state created by `state`.
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
            touches: [ViewId::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            mouse_button: None,
            key_mods: ModifiersState::default(),
            root_id: ViewId { id: 1 },
            focused_id: None,
            window,
            window_title: "rui".into(),
            state_map: HashMap::new(),
            dirty: false,
            enable_dirty: true,
        }
    }

    pub(crate) fn set_dirty(&mut self) {
        if self.enable_dirty {
            self.dirty = true
        }
    }

    pub(crate) fn clear_dirty(&mut self) {
        self.dirty = false;
        for holder in &mut self.state_map.values_mut() {
            holder.dirty = false;
        }
    }

    pub(crate) fn init_state<S: 'static, D: Fn() -> S + 'static>(&mut self, id: ViewId, func: &D) {
        self.state_map.entry(id).or_insert_with(|| StateHolder {
            state: Box::new((func)()),
            dirty: false,
        });
    }

    pub fn get<S>(&self, id: State<S>) -> &S
    where
        S: 'static,
    {
        self.state_map[&id.id].state.downcast_ref::<S>().unwrap()
    }

    pub fn get_mut<S>(&mut self, id: State<S>) -> &mut S
    where
        S: 'static,
    {
        self.set_dirty();

        let mut holder = self.state_map.get_mut(&id.id).unwrap();
        holder.dirty = true;
        holder.state.downcast_mut::<S>().unwrap()
    }
}

impl<S> ops::Index<State<S>> for Context
where
    S: 'static,
{
    type Output = S;

    fn index(&self, index: State<S>) -> &Self::Output {
        self.get(index)
    }
}

impl<S> ops::IndexMut<State<S>> for Context
where
    S: 'static,
{
    fn index_mut(&mut self, index: State<S>) -> &mut Self::Output {
        self.get_mut(index)
    }
}
