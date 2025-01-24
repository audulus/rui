use core::f32;
use std::convert::TryFrom;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use rui::*;

/// Type alias for MIDI note identifiers (0-127)
pub type MidiNoteId = u8;

pub trait MidiNoteIdMethods {
    fn as_note(&self) -> MidiNote;
    fn as_note_kind(&self) -> MidiNoteKind;
    fn as_frequency(&self) -> MidiFrequency;
}

impl MidiNoteIdMethods for MidiNoteId {
    fn as_note(&self) -> MidiNote {
        let note = MidiNoteKind::try_from(*self % 12).unwrap();
        let octave = (*self / 12) as u8;
        MidiNote::new(note, octave)
    }

    fn as_note_kind(&self) -> MidiNoteKind {
        MidiNoteKind::try_from(*self % 12).unwrap()
    }

    fn as_frequency(&self) -> MidiFrequency {
        let freq: MidiFrequency = 440.0 * f32::powf(2.0, (*self as f32 - 69.0) / 12.0);
        freq
    }
}

/// Type alias for MIDI note frequencies in Hz
pub type MidiFrequency = f32;

/// Represents a MIDI note type (C, C#, D, etc.) without octave information
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MidiNoteKind {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl MidiNoteKind {
    /// Converts the MIDI note type to a MIDI note identifier
    pub fn to_midi_note_id(&self) -> MidiNoteId {
        match self {
            MidiNoteKind::C => 0,
            MidiNoteKind::CSharp => 1,
            MidiNoteKind::D => 2,
            MidiNoteKind::DSharp => 3,
            MidiNoteKind::E => 4,
            MidiNoteKind::F => 5,
            MidiNoteKind::FSharp => 6,
            MidiNoteKind::G => 7,
            MidiNoteKind::GSharp => 8,
            MidiNoteKind::A => 9,
            MidiNoteKind::ASharp => 10,
            MidiNoteKind::B => 11,
        }
    }
}

impl TryFrom<MidiNoteId> for MidiNoteKind {
    type Error = ();

    fn try_from(value: MidiNoteId) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MidiNoteKind::C),
            1 => Ok(MidiNoteKind::CSharp),
            2 => Ok(MidiNoteKind::D),
            3 => Ok(MidiNoteKind::DSharp),
            4 => Ok(MidiNoteKind::E),
            5 => Ok(MidiNoteKind::F),
            6 => Ok(MidiNoteKind::FSharp),
            7 => Ok(MidiNoteKind::G),
            8 => Ok(MidiNoteKind::GSharp),
            9 => Ok(MidiNoteKind::A),
            10 => Ok(MidiNoteKind::ASharp),
            11 => Ok(MidiNoteKind::B),
            _ => Err(()),
        }
    }
}

/// Represents a complete MIDI note combining note type and octave
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiNote {
    /// The type of note (C, C#, D, etc.)
    pub note: MidiNoteKind,
    /// The octave number (0-10, where 4 contains middle C)
    pub octave: u8,
}

impl MidiNote {
    /// Creates a new MIDI note with the specified note type and octave
    pub fn new(note: MidiNoteKind, octave: u8) -> Self {
        Self { note, octave }
    }

    /// Returns the MIDI note one octave lower than the current note
    pub fn lower_octave(&self) -> Self {
        if self.octave == 0 {
            Self {
                note: self.note,
                octave: 0,
            }
        } else {
            Self {
                note: self.note,
                octave: self.octave - 1,
            }
        }
    }

    /// Returns the MIDI note one octave higher than the current note
    pub fn higher_octave(&self) -> Self {
        if self.octave == 10 {
            Self {
                note: self.note,
                octave: 10,
            }
        } else {
            Self {
                note: self.note,
                octave: self.octave + 1,
            }
        }
    }

    /// Returns the MIDI note one semitone lower than the current note
    /// If the note is already C, it wraps around to B
    pub fn lower_semitone(&self) -> Self {
        match self.note {
            MidiNoteKind::C => Self {
                note: MidiNoteKind::B,
                octave: self.octave - 1,
            },
            _ => Self {
                note: MidiNoteKind::try_from(self.note as u8 - 1).unwrap(),
                octave: self.octave,
            },
        }
    }

    /// Returns the MIDI note one semitone higher than the current note
    /// If the note is already B, it wraps around to C
    pub fn higher_semitone(&self) -> Self {
        match self.note {
            MidiNoteKind::B => Self {
                note: MidiNoteKind::C,
                octave: self.octave + 1,
            },
            _ => Self {
                note: MidiNoteKind::try_from(self.note as u8 + 1).unwrap(),
                octave: self.octave,
            },
        }
    }

    /// Returns the Audio Frequency of the MIDI note
    /// The formula is 440 * 2^((note - 69) / 12)
    pub fn frequency(&self) -> MidiFrequency {
        let freq: MidiFrequency = 440.0 * f32::powf(2.0, (self.id() as f32 - 69.0) / 12.0);
        freq
    }

    /// Returns the MidiNoteId of the MIDI note
    pub fn id(&self) -> MidiNoteId {
        let note: u8 = match self.note {
            MidiNoteKind::C => 0,
            MidiNoteKind::CSharp => 1,
            MidiNoteKind::D => 2,
            MidiNoteKind::DSharp => 3,
            MidiNoteKind::E => 4,
            MidiNoteKind::F => 5,
            MidiNoteKind::FSharp => 6,
            MidiNoteKind::G => 7,
            MidiNoteKind::GSharp => 8,
            MidiNoteKind::A => 9,
            MidiNoteKind::ASharp => 10,
            MidiNoteKind::B => 11,
        };
        let octave = self.octave as u8;
        octave * 12 + note
    }
}

/// Wrapper for callback functions that handle MIDI note events
struct MidiCallback {
    callback: Box<dyn Fn(MidiNote)>,
}

impl MidiCallback {
    /// Creates a new MidiCallback with the specified callback function
    pub fn new(callback: impl Fn(MidiNote) + 'static) -> Self {
        MidiCallback {
            callback: Box::new(callback),
        }
    }

    /// Executes the stored callback function with the given note
    pub fn run(&self, note: MidiNote) {
        (self.callback)(note);
    }
}

/// Configuration builder for the MIDI keyboard widget
#[derive(Clone)]
pub struct MidiKeyboardConfig {
    /// Number of keys on the keyboard (default: 25)
    num_keys: usize,
    /// Callback triggered when a note begins (from any source)
    on_note_begin: Arc<Mutex<MidiCallback>>,
    /// Callback triggered when a note ends (from any source)
    on_note_end: Arc<Mutex<MidiCallback>>,
}

impl MidiKeyboardConfig {
    /// Creates a new keyboard configuration with default settings
    pub fn new() -> Self {
        Self {
            num_keys: 25,
            on_note_begin: Arc::new(Mutex::new(MidiCallback::new(|_| {}))),
            on_note_end: Arc::new(Mutex::new(MidiCallback::new(|_| {}))),
        }
    }

    /// Sets the number of keys on the keyboard
    pub fn num_keys(mut self, num_keys: usize) -> Self {
        self.num_keys = num_keys;
        self
    }

    /// Sets the callback for note begin events
    pub fn on_note_begin(mut self, callback: impl Fn(MidiNote) + 'static) -> Self {
        self.on_note_begin = Arc::new(Mutex::new(MidiCallback::new(callback)));
        self
    }

    /// Sets the callback for note end events
    pub fn on_note_end(mut self, callback: impl Fn(MidiNote) + 'static) -> Self {
        self.on_note_end = Arc::new(Mutex::new(MidiCallback::new(callback)));
        self
    }

    /// Creates and displays the keyboard widget with the current configuration
    pub fn show(self) -> impl View {
        MidiKeyboard::without_config().show(self)
    }
}

/// Represents the state of a single key on the MIDI keyboard
#[derive(Clone, Copy, Debug)]
struct MidiKeyState {
    /// The musical note associated with this key
    id: MidiNote,
    /// Timestamp of when the key was pressed, None if not pressed
    pressed_at: Option<Instant>,
}

impl MidiKeyState {
    /// Check if the key is currently held
    fn is_held(&self) -> bool {
        self.pressed_at.is_some()
    }

    /// Release the key
    fn release(&mut self) {
        self.pressed_at = None;
    }

    /// Press the key
    fn press(&mut self) {
        if self.pressed_at.is_none() {
            self.pressed_at = Some(Instant::now());
        }
    }
}

/// Maintains the state for the entire MIDI keyboard widget
struct MidiKeyboardState {
    /// State of all keys on the keyboard
    keys: Vec<MidiKeyState>,
    /// Total number of keys
    num_keys: usize,
    /// Number of white keys (used for layout calculations)
    num_white_keys: usize,
    /// Callback for note begin events
    on_note_begin: Arc<Mutex<MidiCallback>>,
    /// Callback for note end events
    on_note_end: Arc<Mutex<MidiCallback>>,
    /// Trigger a re-render when the state changes
    re_render: u32,
    /// Is the user dragging the mouse
    dragging: bool,
    /// Hover position
    hover_pos: Option<LocalPoint>,
    /// Hovered key index
    hovered_key_idx: Option<usize>,
}

impl MidiKeyboardState {
    /// Creates a new keyboard state with the specified configuration
    fn new(config: &MidiKeyboardConfig) -> Self {
        let num_keys = config.num_keys;
        let num_white_keys = Self::calculate_white_key_count(num_keys);
        let start_octave = 4;

        let keys = (0..num_keys)
            .map(|index| {
                let note: MidiNoteKind = MidiNoteKind::try_from(index as u8 % 12).unwrap();
                let octave: u8 = start_octave + (index / 12) as u8;
                MidiKeyState {
                    id: MidiNote { note, octave },
                    pressed_at: None,
                }
            })
            .collect();

        Self {
            keys,
            num_keys,
            num_white_keys,
            on_note_begin: config.on_note_begin.clone(),
            on_note_end: config.on_note_end.clone(),
            re_render: 0,
            dragging: false,
            hover_pos: None,
            hovered_key_idx: None,
        }
    }

    fn re_render(&mut self) {
        self.re_render += 1;
        self.re_render = self.re_render % u32::MAX;
    }

    /// Calculates the total number of white keys based on the total number of keys
    fn calculate_white_key_count(num_keys: usize) -> usize {
        let total_octaves = num_keys / 12;
        let remainder = num_keys % 12;
        total_octaves * 7 + Self::white_key_count_in_remainder(remainder)
    }

    /// Calculates the number of white keys in a partial octave
    fn white_key_count_in_remainder(remainder: usize) -> usize {
        match remainder {
            0 => 0,
            1 | 3 | 4 | 5 | 7 | 8 | 10 | 11 => 1,
            _ => 2,
        }
    }

    /// Press a specific key
    fn press_key(&mut self, idx: usize) {
        if idx < self.keys.len() && !self.keys[idx].is_held() {
            self.keys[idx].press();
            // Trigger note begin callback
            self.on_note_begin.lock().unwrap().run(self.keys[idx].id);
        }
    }

    /// Release all currently held keys
    fn release_all_keys(&mut self) {
        for key in &mut self.keys {
            if key.is_held() {
                // Trigger note end callback
                self.on_note_end.lock().unwrap().run(key.id);
                key.release();
            }
        }
    }
}

/// Main MIDI keyboard widget implementation
#[derive(Default, Clone)]
pub struct MidiKeyboard;

impl MidiKeyboard {
    /// Creates a new keyboard configuration builder
    pub fn new() -> MidiKeyboardConfig {
        MidiKeyboardConfig::new()
    }

    /// Creates a keyboard instance without configuration
    pub fn without_config() -> Self {
        Self::default()
    }

    /// Determines if a key position corresponds to a white key
    fn is_white_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
    }

    /// Determines if a key position corresponds to a black key
    fn is_black_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 1 | 3 | 6 | 8 | 10)
    }

    /// Renders a white key at the specified position
    fn draw_white_key(
        vger: &mut Vger,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        held: bool,
        hovered: bool,
    ) {
        let color = if held {
            vger::Color::new(0.8, 0.8, 0.8, 1.0)
        } else if hovered {
            vger::Color::new(0.85, 0.85, 0.85, 1.0)
        } else {
            vger::Color::new(0.9, 0.9, 0.9, 1.0)
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 0.0, paint_index);
    }

    /// Renders a black key at the specified position
    fn draw_black_key(
        vger: &mut Vger,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        held: bool,
        hovered: bool,
    ) {
        let base_color = vger::Color::new(0.05, 0.05, 0.05, 1.0);
        let color = if held {
            vger::Color::new(0.2, 0.2, 0.2, 1.0)
        } else if hovered {
            vger::Color::new(0.15, 0.15, 0.15, 1.0)
        } else {
            base_color
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 0.0, paint_index);
    }

    /// Displays the keyboard widget with the specified configuration
    pub fn show(self, config: MidiKeyboardConfig) -> impl View {
        state(
            move || MidiKeyboardState::new(&config),
            |s, _| {
                canvas(move |cx, rect, vger| {
                    let white_key_width = rect.width() / cx[s].num_white_keys as f32;
                    let key_height = rect.height();
                    let black_key_height = key_height * 0.6;
                    let black_key_width = white_key_width * 0.7;
                    let mut white_key_count;
                    let mut hovered_key_idx: Option<usize> = None;

                    // Calculate hovered key
                    if let Some(hover_pos) = cx[s].hover_pos {
                        // First check black keys (they're on top)
                        for key_pos in 0..cx[s].num_keys {
                            if Self::is_black_key(key_pos) {
                                let offset = match key_pos % 12 {
                                    1 => 1.0,
                                    3 => 2.0,
                                    6 => 4.0,
                                    8 => 5.0,
                                    10 => 6.0,
                                    _ => continue,
                                };
                                let octave = (key_pos / 12) as f32;
                                let x = (octave * 7.0 + offset) * white_key_width
                                    - (black_key_width / 2.0);
                                if hover_pos.x >= x
                                    && hover_pos.x <= x + black_key_width
                                    && hover_pos.y >= (key_height - black_key_height)
                                    && hover_pos.y <= key_height
                                {
                                    hovered_key_idx = Some(key_pos);
                                }
                            }
                        } // If no black key is hovered, check white keys
                        if hovered_key_idx.is_none() {
                            white_key_count = 0;
                            for key_pos in 0..cx[s].num_keys {
                                if Self::is_white_key(key_pos) {
                                    let x = white_key_count as f32 * white_key_width;
                                    if hover_pos.x >= x && hover_pos.x <= x + white_key_width {
                                        hovered_key_idx = Some(key_pos);
                                    }
                                    white_key_count += 1;
                                }
                            }
                        }
                    }

                    // Reset white key count for drawing
                    white_key_count = 0;

                    // Draw white keys
                    for key_pos in 0..cx[s].num_keys {
                        if Self::is_white_key(key_pos) {
                            let x = white_key_count as f32 * white_key_width;
                            Self::draw_white_key(
                                vger,
                                x,
                                0.0,
                                white_key_width,
                                key_height,
                                cx[s].keys[key_pos].is_held(),
                                hovered_key_idx == Some(key_pos),
                            );
                            white_key_count += 1;
                        }
                    }

                    // Draw black keys
                    for key_pos in 0..cx[s].num_keys {
                        if Self::is_black_key(key_pos) {
                            let offset = match key_pos % 12 {
                                1 => 1.0,
                                3 => 2.0,
                                6 => 4.0,
                                8 => 5.0,
                                10 => 6.0,
                                _ => continue,
                            };
                            let octave = (key_pos / 12) as f32;
                            let x =
                                (octave * 7.0 + offset) * white_key_width - (black_key_width / 2.0);
                            Self::draw_black_key(
                                vger,
                                x,
                                key_height - black_key_height,
                                black_key_width,
                                black_key_height,
                                cx[s].keys[key_pos].is_held(),
                                hovered_key_idx == Some(key_pos),
                            );
                        }
                    }

                    if let Some(idx) = hovered_key_idx {
                        cx[s].hovered_key_idx = Some(idx);
                    }

                    // Instead do state update here instead of in the drag callback
                    // Call press_key when the mouse is down
                    if cx.mouse_buttons.left {
                        if let Some(idx) = cx[s].hovered_key_idx {
                            cx[s].keys[idx].pressed_at = Some(Instant::now());
                            cx[s].press_key(idx);
                        }
                    } else {
                        cx[s].keys.iter_mut().for_each(|key| {
                            if key.is_held() {
                                key.pressed_at = None;
                                key.release();
                            }
                        });
                    }

                    // // Release all keys when the mouse is released
                    // if !cx.mouse_buttons.left {
                    //     cx[s].release_all_keys();
                    // }

                    if cx.mouse_buttons.left {
                        // Hack to re-render when the mouse is down
                        cx[s].re_render();
                    }
                })
                .hover_p(move |cx, pos| {
                    cx[s].hover_pos = Some(pos);
                })
                .hover(move |cx, hover| {
                    if hover {
                        // Hack to re-render when the mouse moves
                        cx[s].re_render();
                    } else {
                        cx[s].hover_pos = None;
                    }
                })
                // .drag(move |cx, _, gesture_state, mouse_button| {
                //     if mouse_button == Some(MouseButton::Left) {
                //         match gesture_state {
                //             GestureState::Began => {
                //                 cx[s].dragging = true;
                //                 if let Some(idx) = cx[s].hovered_key_idx {
                //                     cx[s].press_key(idx);
                //                 }
                //             }
                //             GestureState::Changed => {
                //                 // Handle mouse drag or click
                //                 if let Some(idx) = cx[s].hovered_key_idx {
                //                     cx[s].press_key(idx);
                //                 }
                //             }
                //             GestureState::Ended => {
                //                 cx[s].dragging = false;
                //                 cx[s].release_all_keys();
                //             }
                //             _ => {}
                //         }
                //     }
                // })
            },
        )
    }
}

// Implement Default for MidiKeyboardConfig
impl Default for MidiKeyboardConfig {
    fn default() -> Self {
        Self::new()
    }
}
