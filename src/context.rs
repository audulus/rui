
use std::collections::HashMap;
use std::any::Any;

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
pub struct ViewID {
    path: [u16; 32],
    len: usize
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
    pub state_map: HashMap<ViewID, Box<dyn Any>>
}

impl Context {

    pub fn new() -> Self {
        Self {
            state_map: HashMap::new()
        }
    }
}