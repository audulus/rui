use core::f32;
use std::collections::HashSet;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;

use rui::*;

/// Type alias for MIDI note identifiers (0-127)
pub type MidiNoteId = u8;

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

/// Extension methods for MIDI note conversion and manipulation
#[allow(dead_code)]
pub trait MidiNoteIdMethods {
    /// Converts a MIDI note ID to a complete MIDI note with note type and octave
    fn as_note(&self) -> MidiNote;

    /// Extracts the note type from a MIDI note ID
    fn as_note_kind(&self) -> MidiNoteKind;

    /// Calculates the frequency in Hz for the given MIDI note ID
    fn as_frequency(&self) -> MidiFrequency;
}

impl MidiNoteIdMethods for MidiNoteId {
    fn as_note(&self) -> MidiNote {
        let note = MidiNoteKind::try_from(*self % 12).unwrap();
        let octave = *self / 12;
        MidiNote::new(note, octave)
    }

    fn as_note_kind(&self) -> MidiNoteKind {
        self.as_note().note
    }

    fn as_frequency(&self) -> MidiFrequency {
        self.as_note().frequency()
    }
}

impl MidiNoteKind {
    /// Converts the MIDI note type to its corresponding MIDI note identifier
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
        match value % 12 {
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
    pub octave: MidiNoteId,
}

impl MidiNote {
    /// Creates a new MIDI note with the specified note type and octave
    ///
    /// # Arguments
    /// * `note` - The type of note (C, C#, D, etc.)
    /// * `octave` - The octave number (0-10)
    ///
    /// # Returns
    /// A new MidiNote instance
    pub fn new(note: MidiNoteKind, octave: MidiNoteId) -> Self {
        Self {
            note,
            octave: octave.clamp(0, 10),
        }
    }

    /// Returns the MIDI note one octave lower
    pub fn lower_octave(&self) -> Self {
        Self {
            note: self.note,
            octave: self.octave.saturating_sub(1),
        }
    }

    /// Returns the MIDI note one octave higher
    pub fn higher_octave(&self) -> Self {
        Self {
            note: self.note,
            octave: (self.octave + 1).min(10),
        }
    }

    /// Returns the MIDI note one semitone lower
    pub fn lower_semitone(&self) -> Self {
        match self.note {
            MidiNoteKind::C => Self {
                note: MidiNoteKind::B,
                octave: self.octave.saturating_sub(1),
            },
            _ => Self {
                note: MidiNoteKind::try_from(self.note as MidiNoteId - 1).unwrap(),
                octave: self.octave,
            },
        }
    }

    /// Returns the MIDI note one semitone higher
    pub fn higher_semitone(&self) -> Self {
        match self.note {
            MidiNoteKind::B => Self {
                note: MidiNoteKind::C,
                octave: (self.octave + 1).min(10),
            },
            _ => Self {
                note: MidiNoteKind::try_from(self.note as MidiNoteId + 1).unwrap(),
                octave: self.octave,
            },
        }
    }

    /// Calculates the audio frequency of the MIDI note in Hz
    pub fn frequency(&self) -> MidiFrequency {
        440.0 * f32::powf(2.0, (self.id() as f32 - 69.0) / 12.0)
    }

    /// Returns the MIDI note identifier (0-127)
    pub fn id(&self) -> MidiNoteId {
        let note_id = self.note.to_midi_note_id();
        self.octave * 12 + note_id
    }
}

/// MIDI note event with velocity and timestamp
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiNoteEvent {
    /// The MIDI note details
    pub note: MidiNote,
    /// Note velocity (0-127)
    pub velocity: MidiNoteId,
    /// Timestamp of the note event
    pub timestamp: Instant,
}

/// Configuration builder for MIDI keyboard with advanced customization options
#[derive(Clone)]
pub struct MidiKeyboardConfig {
    start_octave: MidiNoteId,
    num_keys: MidiNoteId,
    max_simultaneous_keys: MidiNoteId,
    note_begin_handler: Option<Arc<dyn Fn(MidiNoteEvent) + Send + Sync>>,
    note_end_handler: Option<Arc<dyn Fn(MidiNoteEvent) + Send + Sync>>,
}

impl MidiKeyboardConfig {
    /// Creates a new MIDI keyboard configuration with default settings
    ///
    /// Default configuration:
    /// - Start octave: 4 (middle C)
    /// - Number of keys: 25
    /// - Maximum simultaneous keys: 10
    /// - No note begin/end handlers
    pub fn new() -> Self {
        Self {
            start_octave: 4,
            num_keys: 25,
            max_simultaneous_keys: 10,
            note_begin_handler: None,
            note_end_handler: None,
        }
    }

    /// Sets the starting octave for the keyboard
    ///
    /// # Arguments
    /// * `octave` - Octave to start from (0-10)
    pub fn start_octave(mut self, octave: MidiNoteId) -> Self {
        self.start_octave = octave.clamp(0, 10);
        self
    }

    /// Sets the total number of keys on the keyboard
    ///
    /// # Arguments
    /// * `keys` - Number of keys (1-88)
    pub fn num_keys(mut self, keys: MidiNoteId) -> Self {
        self.num_keys = keys.clamp(1, 88);
        self
    }

    /// Sets the maximum number of keys that can be pressed simultaneously
    ///
    /// # Arguments
    /// * `max_keys` - Maximum number of simultaneous key presses
    pub fn max_simultaneous_keys(mut self, max_keys: MidiNoteId) -> Self {
        self.max_simultaneous_keys = max_keys;
        self
    }

    /// Sets a handler for when a note begins (key press)
    ///
    /// # Arguments
    /// * `handler` - Callback function for note begin events
    pub fn on_note_begin(
        mut self,
        handler: impl Fn(MidiNoteEvent) + Send + Sync + 'static,
    ) -> Self {
        self.note_begin_handler = Some(Arc::new(handler));
        self
    }

    /// Sets a handler for when a note ends (key release)
    ///
    /// # Arguments
    /// * `handler` - Callback function for note end events
    pub fn on_note_end(mut self, handler: impl Fn(MidiNoteEvent) + Send + Sync + 'static) -> Self {
        self.note_end_handler = Some(Arc::new(handler));
        self
    }

    /// Renders and displays the MIDI keyboard
    pub fn show(self) -> impl View {
        MidiKeyboard::show(self)
    }
}

/// Keyboard state management with advanced features
struct MidiKeyboardState {
    keys: Vec<Option<MidiNoteEvent>>,
    pressed_keys: HashSet<MidiNoteId>,
    config: MidiKeyboardConfig,
    last_interaction: Instant,
    keyboard_layout: Vec<(f32, f32, bool)>, // (x, width, is_black_key)
}

impl MidiKeyboardState {
    fn new(config: MidiKeyboardConfig) -> Self {
        let keyboard_layout = Self::calculate_keyboard_layout(config.num_keys);
        Self {
            keys: vec![None; config.num_keys as usize],
            pressed_keys: HashSet::new(),
            config,
            last_interaction: Instant::now(),
            keyboard_layout,
        }
    }

    fn calculate_keyboard_layout(num_keys: MidiNoteId) -> Vec<(f32, f32, bool)> {
        let mut layout = Vec::new();
        let mut white_key_count = 0;
        let black_key_positions = [1, 3, 6, 8, 10]; // Relative positions of black keys

        for key_pos in 0..num_keys {
            let key_in_octave = key_pos % 12;

            let is_black_key = black_key_positions.contains(&key_in_octave);
            let x = if is_black_key {
                // Precise black key positioning
                white_key_count as f32 - 0.3
            } else {
                let current_white_key = white_key_count;
                white_key_count += 1;
                current_white_key as f32
            };

            layout.push((
                x,
                if is_black_key { 0.6 } else { 1.0 }, // Narrower black keys
                is_black_key,
            ));
        }

        layout
    }

    fn num_white_keys(&self) -> usize {
        self.keyboard_layout
            .iter()
            .filter(|&&(_, _, is_black_key)| !is_black_key)
            .count()
    }

    fn press_key(&mut self, index: MidiNoteId, velocity: MidiNoteId) -> Result<(), &'static str> {
        if self.pressed_keys.len() as MidiNoteId >= self.config.max_simultaneous_keys {
            return Err("Maximum simultaneous keys reached");
        }

        if self.pressed_keys.contains(&index) {
            return Err("Key already pressed");
        }

        let note_event = MidiNoteEvent {
            note: self.calculate_note_for_index(index as usize),
            velocity,
            timestamp: Instant::now(),
        };

        self.keys[index as usize] = Some(note_event);
        self.pressed_keys.insert(index);
        self.last_interaction = Instant::now();

        if let Some(handler) = &self.config.note_begin_handler {
            handler(note_event);
        }

        Ok(())
    }

    fn release_key(&mut self, index: MidiNoteId) -> Result<(), &'static str> {
        if let Some(note_event) = self.keys[index as usize].take() {
            self.pressed_keys.remove(&index);
            self.last_interaction = Instant::now();

            if let Some(handler) = &self.config.note_end_handler {
                handler(note_event);
            }

            Ok(())
        } else {
            Err("No active key to release")
        }
    }

    fn calculate_note_for_index(&self, index: usize) -> MidiNote {
        let note_kind = MidiNoteKind::try_from((index % 12) as MidiNoteId).unwrap();
        let octave = self.config.start_octave + (index / 12) as MidiNoteId;
        MidiNote::new(note_kind, octave)
    }

    fn release_all_keys(&mut self) {
        let current_pressed = self.pressed_keys.clone();
        for index in current_pressed {
            let _ = self.release_key(index);
        }
    }
}

/// Primary MIDI keyboard implementation
pub struct MidiKeyboard;

impl MidiKeyboard {
    /// Creates a new MIDI keyboard configuration with default settings
    ///
    /// Equivalent to `MidiKeyboardConfig::new()`
    pub fn new() -> MidiKeyboardConfig {
        MidiKeyboardConfig::new()
    }

    /// Renders the MIDI keyboard based on the provided configuration
    ///
    /// # Arguments
    /// * `config` - Configuration for the MIDI keyboard
    pub fn show(config: MidiKeyboardConfig) -> impl View {
        state(
            move || MidiKeyboardState::new(config.clone()),
            |s, _| {
                canvas(move |cx, rect, vger| {
                    let total_white_keys = cx[s].num_white_keys();
                    let white_key_width = rect.width() / total_white_keys as f32;
                    let key_height = rect.height();
                    let black_key_height = key_height * 0.6;

                    let mut hovered_key_idx: Option<MidiNoteId> = None;

                    // Calculate hovered key
                    if let Some(hover_pos) = cx.mouse_position {
                        for (index, (x, width, is_black_key)) in
                            cx[s].keyboard_layout.iter().enumerate()
                        {
                            let key_x = x * white_key_width;
                            let key_width = white_key_width * width;
                            let key_y = if *is_black_key {
                                key_height - black_key_height
                            } else {
                                0.0
                            };
                            let key_height_check = if *is_black_key {
                                black_key_height
                            } else {
                                key_height
                            };

                            if hover_pos.x >= key_x
                                && hover_pos.x <= key_x + key_width
                                && hover_pos.y >= key_y
                                && hover_pos.y <= key_y + key_height_check
                            {
                                // Prioritize black keys (they're rendered on top)
                                if *is_black_key {
                                    hovered_key_idx = Some(index as MidiNoteId);
                                    break;
                                } else if hovered_key_idx.is_none() {
                                    hovered_key_idx = Some(index as MidiNoteId);
                                }
                            }
                        }
                    }

                    // Draw white keys first (underneath)
                    for (index, (x, width, is_black_key)) in
                        cx[s].keyboard_layout.iter().enumerate()
                    {
                        if !is_black_key {
                            let key_x = x * white_key_width;
                            let key_width = white_key_width * width;

                            Self::draw_white_key(
                                vger,
                                key_x,
                                0.0,
                                key_width,
                                key_height,
                                cx[s].keys[index].is_some(),
                                hovered_key_idx == Some(index as MidiNoteId),
                            );
                        }
                    }

                    // Draw black keys on top
                    for (index, (x, width, is_black_key)) in
                        cx[s].keyboard_layout.iter().enumerate()
                    {
                        if *is_black_key {
                            let key_x = x * white_key_width;
                            let key_width = white_key_width * width;

                            Self::draw_black_key(
                                vger,
                                key_x,
                                key_height - black_key_height,
                                key_width,
                                black_key_height,
                                cx[s].keys[index].is_some(),
                                hovered_key_idx == Some(index as MidiNoteId),
                            );
                        }
                    }

                    // Handle mouse interactions
                    if cx.mouse_buttons.left {
                        if let Some(idx) = hovered_key_idx {
                            let _ = cx[s].press_key(idx, 100);
                        }
                    } else {
                        cx[s].release_all_keys();
                    }
                })
                .hover(move |cx, is_hovering| {
                    if !is_hovering {
                        cx[s].release_all_keys();
                    }
                })
            },
        )
    }

    // Draw methods remain the same as in the original implementation
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
            vger::Color::new(1.0, 1.0, 1.0, 1.0) // Pure white for more realistic look
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));

        vger.fill_rect(rect, 0.0, paint_index);
    }

    fn draw_black_key(
        vger: &mut Vger,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        held: bool,
        hovered: bool,
    ) {
        let base_color = vger::Color::new(0.1, 0.1, 0.1, 1.0);
        let color = if held {
            vger::Color::new(0.3, 0.3, 0.3, 1.0)
        } else if hovered {
            vger::Color::new(0.2, 0.2, 0.2, 1.0)
        } else {
            base_color
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 2.0, paint_index); // Add rounded corners
    }
}
