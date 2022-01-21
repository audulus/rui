
use crate::*;
use std::collections::HashMap;

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash, Debug)]
struct ViewID {
    path: [u16; 32],
    len: usize
}

struct Context {
    state_map: HashMap<ViewID, Box<dyn AnyState>>
}