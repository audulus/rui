use crate::*;

/// Type of event.
#[derive(Clone, Debug)]
pub(crate) enum EventKind {
    TouchBegin { id: usize },
    TouchMove { id: usize },
    TouchEnd { id: usize },
    Command(String),
    Key(KeyPress, ModifiersState),
}

/// Used internally for event processing.
#[derive(Clone, Debug)]
pub struct Event {
    pub(crate) kind: EventKind,
    pub(crate) position: LocalPoint,
}
