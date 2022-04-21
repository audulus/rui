use crate::*;

/// Type of event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum EventKind {
    TouchBegin { id: usize },
    TouchMove { id: usize },
    TouchEnd { id: usize },
    Command(String),
    Key(KeyPress),
    Anim,
}

/// Used internally for event processing.
#[derive(Clone, Debug)]
pub struct Event {
    pub(crate) kind: EventKind,
    pub(crate) position: LocalPoint,
}
