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

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewID {
    id: u64,
}

impl ViewID {
    pub fn child<T: Hash>(&self, value: &T) -> Self {
        let mut hasher = DefaultHasher::new();
        hasher.write_u64(self.id);
        value.hash(&mut hasher);
        Self {
            id: hasher.finish(),
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

/// The Context stores all UI state. A user of the library
/// shouldn't have to interact with it directly.
pub struct Context {

    /// Map for `state`.
    state_map: HashMap<ViewID, Box<dyn Any>>,

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
}

impl Context {
    pub fn new() -> Self {
        Self {
            state_map: HashMap::new(),
            layout: HashMap::new(),
            vger: None,
            touches: [ViewID::default(); 16],
            starts: [LocalPoint::zero(); 16],
            previous_position: [LocalPoint::zero(); 16],
            root_id: ViewID::default(),
        }
    }

    pub fn with_state<S: Clone + 'static, R, F: Fn(State<S>, &mut Self) -> R>(
        &mut self,
        default: S,
        id: ViewID,
        f: F,
    ) -> R {
        let newstate = Box::new(State::new(default));
        let s = self.state_map.entry(id).or_insert(newstate);

        if let Some(state) = s.downcast_ref::<State<S>>() {
            f(state.clone(), self)
        } else {
            panic!("state has wrong type")
        }
    }

    pub fn with_state_vger<S: Clone + 'static, R, F: Fn(State<S>, &mut Self, &mut VGER) -> R>(
        &mut self,
        vger: &mut VGER,
        default: S,
        id: ViewID,
        f: F,
    ) -> R {
        let newstate = Box::new(State::new(default));
        let s = self.state_map.entry(id).or_insert(newstate);

        if let Some(state) = s.downcast_ref::<State<S>>() {
            f(state.clone(), self, vger)
        } else {
            panic!("state has wrong type")
        }
    }
}
