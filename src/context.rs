
use crate::*;
use std::collections::HashMap;

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
struct ViewID {
    path: [u16; 32],
    len: usize
}

impl ViewID {
    fn child(&self, index: u16) -> Self {
        let mut c = *self;
        assert!(c.len < 32);
        c.path[c.len] = index;
        c.len += 1;
        c
    }
}

struct Context {
    state_map: HashMap<ViewID, Box<dyn AnyState>>
}

impl Context {

    fn new() -> Self {
        Self {
            state_map: HashMap::new()
        }
    }
}