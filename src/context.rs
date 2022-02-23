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

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewID {
    path: [u16; 32],
    len: usize,
}

impl ViewID {
    pub fn child(&self, index: u16) -> Self {
        let mut c = *self;
        assert!(c.len < 32);
        c.path[c.len] = index;
        c.len += 1;
        c
    }
}

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct LayoutBox {
    pub rect: LocalRect,
    pub offset: LocalOffset,
}

pub struct Context {
    state_map: HashMap<ViewID, Box<dyn Any>>,
    pub layout: HashMap<ViewID, LayoutBox>,
    pub vger: Option<VGER>,
    pub touches: [ViewID; 16],
    pub starts: [LocalPoint; 16],
}

impl Context {
    pub fn new() -> Self {
        Self {
            state_map: HashMap::new(),
            layout: HashMap::new(),
            vger: None,
            touches: [ViewID::default(); 16],
            starts: [LocalPoint::zero(); 16]
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
