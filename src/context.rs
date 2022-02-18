use crate::*;
use euclid::*;
use std::any::Any;
use std::collections::HashMap;

pub struct LocalSpace;
pub type LocalRect = Rect<f32, LocalSpace>;
pub type LocalOffset = Vector2D<f32, LocalSpace>;
pub type LocalSize = Size2D<f32, LocalSpace>;
pub type LocalPoint = Point2D<f32, LocalSpace>;

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

#[derive(Copy, Clone, Default, PartialEq, Debug)]
pub struct BoxConstraint {
    pub min_size: LocalSize,
    pub max_size: LocalSize,
}

pub struct Context {
    state_map: HashMap<ViewID, Box<dyn Any>>,
    pub layout: HashMap<ViewID, LayoutBox>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            state_map: HashMap::new(),
            layout: HashMap::new(),
        }
    }

    pub fn with_state<S: Clone + 'static, F: Fn(State<S>, &mut Self)>(
        &mut self,
        default: S,
        id: ViewID,
        f: F,
    ) {
        let newstate = Box::new(State::new(default));
        let s = self.state_map.entry(id).or_insert(newstate);

        if let Some(state) = s.downcast_ref::<State<S>>() {
            f(state.clone(), self)
        } else {
            panic!("state has wrong type")
        }
    }

    pub fn with_state_vger<S: Clone + 'static, F: Fn(State<S>, &mut Self, &mut VGER)>(
        &mut self,
        vger: &mut VGER,
        default: S,
        id: ViewID,
        f: F,
    ) {
        let newstate = Box::new(State::new(default));
        let s = self.state_map.entry(id).or_insert(newstate);

        if let Some(state) = s.downcast_ref::<State<S>>() {
            f(state.clone(), self, vger)
        } else {
            panic!("state has wrong type")
        }
    }
}
