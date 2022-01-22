use std::any::Any;
use std::collections::HashMap;
use crate::*;

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

pub struct Context {
    pub state_map: HashMap<ViewID, Box<dyn Any>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            state_map: HashMap::new(),
        }
    }

    pub fn with_state<S: Clone + 'static, F: Fn(State<S>, &mut Self)>(&mut self, default: S, id: ViewID, f: F) {

        let newstate = Box::new(State::new(default));
        let s = self.state_map.entry(id).or_insert(newstate);

        if let Some(state) = s.downcast_ref::<State<S>>() {
            f(state.clone(), self)
        } else {
            panic!("state has wrong type")
        }
    }
}
