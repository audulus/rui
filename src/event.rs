use crate::*;

#[derive(Clone, Debug)]
pub enum EventKind {
    TouchBegin { id: usize },
    TouchMove { id: usize },
    TouchEnd { id: usize },
    Command(String),
    Key(KeyPress, ModifiersState),
}

#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub position: LocalPoint,
}