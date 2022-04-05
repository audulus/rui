use crate::*;
use euclid::*;
use std::any::Any;
use std::collections::HashMap;

pub type LocalSpace = vger::defs::LocalSpace;
pub type WorldSpace = vger::defs::WorldSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;
pub type WorldRect = Rect<f32, WorldSpace>;
pub type WorldPoint = Point2D<f32, WorldSpace>;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use tao::event_loop::EventLoopProxy;

/// `ViewID` is a unique identifier for a view. We're using a u64 and hashing
/// under the assumption there won't be collsions. The underlying u64 is a function
/// of the path down the view tree.
#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewID {
    id: u64,
}

impl ViewID {
    /// Computes the ID for a child using a hashable value. For views
    /// which don't have dynamic children (e.g. `vstack` etc.) the value
    /// will be the integer index of the child. Dynamic
    /// views (e.g. `list`) will hash an item identifier.
    pub fn child<T: Hash>(&self, value: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.id);
        value.hash(&mut hasher);
        Self {
            id: hasher.finish(),
        }
    }

    pub fn access_id(&self) -> accesskit::NodeId {
        accesskit::NodeId(std::num::NonZeroU64::new(self.id).unwrap())
    }
}

pub const DEBUG_LAYOUT: bool = false;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

// This could use a better name.
pub struct Dirty {
    pub dirty: bool,
    pub event_loop_proxy: Option<EventLoopProxy<()>>
}

impl Dirty {
    pub fn new(event_loop_proxy: Option<EventLoopProxy<()>>) -> Self {
        Dirty{ dirty: false, event_loop_proxy }
    }
}

/// Restricts what we can store in a StateMap (instead of just using Any)
pub trait AnyState {
    /// So we can downcast.
    fn as_any(&self) -> &dyn Any;
}

pub type StateMap = HashMap<ViewID, Box<dyn AnyState>>;

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {
    /// Map for `state`.
    pub state_map: StateMap,

    /// Layout information for all views.
    pub layout: HashMap<ViewID, LayoutBox>,

    /// GPU renderer.
    pub vger: Option<VGER>,

    /// Which views each touch (or mouse pointer) is interacting with.
    pub touches: [ViewID; 16],

    /// Points at which touches (or click-drags) started.
    pub starts: [LocalPoint; 16],

    /// Previous touch/mouse positions.
    pub previous_position: [LocalPoint; 16],

    /// The root view ID. This should be randomized for security reasons.
    pub root_id: ViewID,

    /// The view that has the keybord focus.
    pub focused_id: Option<ViewID>,

    /// Did state change?
    pub dirty: Arc<Mutex<Dirty>>,
}

impl Context {
    pub fn new(event_loop_proxy: Option<EventLoopProxy<()>>) -> Self {
        Self {
            state_map: HashMap::new(),
            layout: HashMap::new(),
            vger: None,
            touches: [ViewID::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            root_id: ViewID::default(),
            focused_id: None,
            dirty: Arc::new(Mutex::new(Dirty::new(event_loop_proxy))),
        }
    }

    pub fn with_state<S: Clone + 'static, R, F: Fn(State<S>, &mut Self) -> R>(
        &mut self,
        default: S,
        id: ViewID,
        f: F,
    ) -> R {
        let d = self.dirty.clone();
        let s = self
            .state_map
            .entry(id)
            .or_insert_with(|| Box::new(State::new(default, d)));

        if let Some(state) = s.as_any().downcast_ref::<State<S>>() {
            f(state.clone(), self)
        } else {
            panic!("state has wrong type")
        }
    }

    pub fn with_state_mut<S: Clone + 'static, R, F: FnMut(State<S>, &mut Self) -> R>(
        &mut self,
        default: S,
        id: ViewID,
        f: &mut F,
    ) -> R {
        let d = self.dirty.clone();
        let s = self
            .state_map
            .entry(id)
            .or_insert_with(|| Box::new(State::new(default, d)));

        if let Some(state) = s.as_any().downcast_ref::<State<S>>() {
            f(state.clone(), self)
        } else {
            panic!("state has wrong type")
        }
    }

    pub fn with_state_aux<S: Clone + 'static, T, R, F: Fn(State<S>, &mut Self, &mut T) -> R>(
        &mut self,
        default: S,
        id: ViewID,
        aux: &mut T,
        f: F,
    ) -> R {
        let d = self.dirty.clone();
        let s = self
            .state_map
            .entry(id)
            .or_insert_with(|| Box::new(State::new(default, d)));

        if let Some(state) = s.as_any().downcast_ref::<State<S>>() {
            f(state.clone(), self, aux)
        } else {
            panic!("state has wrong type")
        }
    }
}
