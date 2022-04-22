use crate::*;

/// User interface event.
#[derive(Clone, Debug)]
pub enum Event {
    /// Touch event, or mouse down.
    TouchBegin {
        /// Identifies a touch so we can track it.
        id: usize,
        position: LocalPoint,
    },

    /// Touch moved or mouse moved while down.
    TouchMove {
        /// Identifies a touch so we can track it.
        id: usize,
        position: LocalPoint,
    },

    /// Touch went up or mouse button released.
    TouchEnd {
        /// Identifies a touch so we can track it.
        id: usize,
        position: LocalPoint,
    },

    /// Menu command.
    Command(String),

    /// Key press.
    Key(KeyPress),

    /// Animation.
    Anim,
}

impl Event {
    pub fn offset(&self, offset: LocalOffset) -> Event {
        let mut event = self.clone();
        match &mut event {
            Event::TouchBegin{ id: _, position} => *position += offset,
            Event::TouchMove{ id: _, position} => *position += offset,
            Event::TouchEnd{ id: _, position} => *position += offset,
            _ => (),
        }
        event
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MouseButton {
    Left,
    Right,
    Center
}
