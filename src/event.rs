use std::sync::Arc;

use crate::*;

/// User interface event.
#[derive(Clone, PartialEq, Debug)]
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
        delta: LocalOffset,
    },

    /// Touch went up or mouse button released.
    TouchEnd {
        /// Identifies a touch so we can track it.
        id: usize,
        position: LocalPoint,
    },

    /// Called when the mouse gets outside the window
    MouseLeftWindow,

    /// Menu command.
    Command(Arc<str>),

    /// Key press.
    Key(Key),

    /// Key released.
    KeyReleased(Key),

    /// Animation.
    Anim,
}

impl Event {
    pub fn offset(&self, offset: LocalOffset) -> Event {
        let mut event = self.clone();
        match &mut event {
            Event::TouchBegin { position, .. } => *position += offset,
            Event::TouchMove { position, .. } => *position += offset,
            Event::TouchEnd { position, .. } => *position += offset,
            _ => (),
        }
        event
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Center,
}
#[derive(Copy, Clone, Debug, Default)]
pub struct MouseButtons {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct KeyboardModifiers {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub command: bool,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ElementState {
    Pressed,
    Released,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Key {
    Character(char),

    Enter,
    Tab,
    Space,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    ArrowUp,
    End,
    Home,
    PageDown,
    PageUp,
    Backspace,
    Delete,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
}

#[derive(Clone, Debug, Eq, PartialEq, Copy)]
pub enum HotKey {
    KeyA,
    KeyB,
    KeyC,
    KeyD,
    KeyE,
    KeyF,
    KeyG,
    KeyH,
    KeyI,
    KeyJ,
    KeyK,
    KeyL,
    KeyM,
    KeyN,
    KeyO,
    KeyP,
    KeyQ,
    KeyR,
    KeyS,
    KeyT,
    KeyU,
    KeyV,
    KeyW,
    KeyX,
    KeyY,
    KeyZ,
}
