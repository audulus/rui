use crate::*;

/// Type of event.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EventKind {
    /// Touch event, or mouse down.
    TouchBegin {
        /// Identifies a touch so we can track it.
        id: usize,
    },

    /// Touch moved or mouse moved while down.
    TouchMove {
        /// Identifies a touch so we can track it.
        id: usize,
    },

    /// Touch went up or mouse button released.
    TouchEnd {
        /// Identifies a touch so we can track it.
        id: usize,
    },

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
