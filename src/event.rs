use crate::*;

/// Type of event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventKind {

    /// Touch event, or mouse down.
    TouchBegin { id: usize },

    /// Touch moved or mouse moved while down.
    TouchMove { id: usize },

    /// Touch went up or mouse button released.
    TouchEnd { id: usize },

    /// Menu command.
    Command(String),

    /// Key press.
    Key(KeyPress),

    /// Animation.
    Anim,
}

/// User interface event.
#[derive(Clone, Debug)]
pub struct Event {
    pub kind: EventKind,
    pub position: LocalPoint,
}
