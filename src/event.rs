use crate::*;

/// Type of event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
    TouchBegin { id: usize },
    TouchMove { id: usize },
    TouchEnd { id: usize },
    Command(String),
    Key(KeyPress),
    Anim,
}

/// User interface event.
#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub position: LocalPoint,
}
