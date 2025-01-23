use core::f32;
use std::convert::TryFrom;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use rui::*;

/// Type alias for MIDI note identifiers (0-127)
pub type MidiNoteId = u8;
/// Type alias for MIDI note frequencies in Hz
pub type MidiFrequency = f32;

/// Converts a MidiNote to its corresponding frequency in Hz
/// Uses the standard MIDI frequency formula: f = 440 * 2^((n-69)/12)
/// where n is the MIDI note number and 440Hz is A4 (concert pitch)
impl TryFrom<MidiNote> for MidiFrequency {
    type Error = ();

    fn try_from(value: MidiNote) -> Result<Self, Self::Error> {
        let note_freqs = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];

        let freq = 440.0
            * f32::powf(
                2.0,
                (value.octave as f32 - 4.0) + note_freqs[value.note as usize] / 12.0,
            );
        Ok(freq)
    }
}

/// Represents a MIDI note type (C, C#, D, etc.) without octave information
/// These represent the 12 standard notes in Western music
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MidiNoteType {
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

/// Represents a complete MIDI note combining note type and octave
/// Standard MIDI uses octaves from 0-10, where middle C is in octave 4
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiNote {
    /// The type of note (C, C#, D, etc.)
    pub note: MidiNoteType,
    /// The octave number (0-10, where 4 contains middle C)
    pub octave: u8,
}

/// Converts a MidiNote to its corresponding MIDI note number (0-127)
/// The conversion follows the standard MIDI specification where:
/// - Middle C (C4) = 60
/// - Each octave has 12 notes
/// - Note numbers increase by 1 for each semitone
impl TryFrom<MidiNote> for MidiNoteId {
    type Error = ();

    fn try_from(value: MidiNote) -> Result<Self, Self::Error> {
        let note: u8 = match value.note {
            MidiNoteType::C => 0,
            MidiNoteType::CSharp => 1,
            MidiNoteType::D => 2,
            MidiNoteType::DSharp => 3,
            MidiNoteType::E => 4,
            MidiNoteType::F => 5,
            MidiNoteType::FSharp => 6,
            MidiNoteType::G => 7,
            MidiNoteType::GSharp => 8,
            MidiNoteType::A => 9,
            MidiNoteType::ASharp => 10,
            MidiNoteType::B => 11,
        };

        let octave = value.octave as u8;
        Ok(octave * 12 + note)
    }
}

/// Converts a MIDI note number (0-11) to its corresponding note type
/// This conversion handles the mapping between numbers and musical notes
/// within a single octave
impl TryFrom<MidiNoteId> for MidiNoteType {
    type Error = ();

    fn try_from(value: MidiNoteId) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(MidiNoteType::C),
            1 => Ok(MidiNoteType::CSharp),
            2 => Ok(MidiNoteType::D),
            3 => Ok(MidiNoteType::DSharp),
            4 => Ok(MidiNoteType::E),
            5 => Ok(MidiNoteType::F),
            6 => Ok(MidiNoteType::FSharp),
            7 => Ok(MidiNoteType::G),
            8 => Ok(MidiNoteType::GSharp),
            9 => Ok(MidiNoteType::A),
            10 => Ok(MidiNoteType::ASharp),
            11 => Ok(MidiNoteType::B),
            _ => Err(()),
        }
    }
}

/// Wrapper for callback functions that handle MIDI note events
/// Provides a type-safe way to store and execute note event callbacks
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
/// Allows customization of keyboard properties and event handlers
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
    pub fn on_note_begin(mut self, on_note_change: impl Fn(MidiNote) + 'static) -> Self {
        self.on_note_begin = Arc::new(Mutex::new(MidiCallback::new(on_note_change)));
        self
    }

    /// Sets the callback for note end events
    pub fn on_note_end(mut self, on_note_change: impl Fn(MidiNote) + 'static) -> Self {
        self.on_note_end = Arc::new(Mutex::new(MidiCallback::new(on_note_change)));
        self
    }

    /// Creates and displays the keyboard widget with the current configuration
    pub fn show(self) -> impl View {
        MidiKeyboard::without_config().show(self)
    }
}

/// Main MIDI keyboard widget implementation
/// Provides a visual piano keyboard interface that responds to mouse input
#[derive(Default, Clone)]
pub struct MidiKeyboard;

impl MidiKeyboard {
    /// Creates a new keyboard configuration builder
    pub fn new() -> MidiKeyboardConfig {
        MidiKeyboardConfig::new()
    }

    /// Creates a keyboard instance without configuration
    /// Typically used internally by the configuration builder
    pub fn without_config() -> Self {
        Self::default()
    }

    /// Renders a white key at the specified position
    /// Handles normal, hover, and pressed states with appropriate coloring
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
            vger::Color::new(0.85, 0.85, 0.85, 1.0) // Slightly darker than normal for hover effect
        } else {
            vger::Color::new(0.9, 0.9, 0.9, 1.0)
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 0.0, paint_index);
    }

    /// Renders a black key at the specified position
    /// Handles normal, hover, and pressed states with appropriate coloring
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
            vger::Color::new(0.15, 0.15, 0.15, 1.0) // Slightly lighter than normal for hover effect
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
                        }

                        // If no black key is hovered, check white keys
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
                                cx[s].keys[key_pos].held.is_some(),
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
                                cx[s].keys[key_pos].held.is_some(),
                                hovered_key_idx == Some(key_pos),
                            );
                        }
                    }

                    if let Some(idx) = hovered_key_idx {
                        cx[s].hovered_key_idx = Some(idx);
                    }
                })
                .hover_p(move |cx, pos| {
                    cx[s].hover_pos = Some(pos);
                })
                .hover(move |cx, hover| {
                    if hover {
                        // Hack for now to re-render when the mouse moves.
                        cx[s].re_render += 1;
                        cx[s].re_render = cx[s].re_render % u32::MAX;
                    } else {
                        cx[s].hover_pos = None;
                    }
                })
                .drag(move |cx, _, gesture_state, mouse_button| {
                    if mouse_button == Some(MouseButton::Left) {
                        match gesture_state {
                            GestureState::Began => {
                                cx[s].dragging = true;
                                println!("Dragging started");
                            }
                            GestureState::Changed => {
                                // Handle mouse drag or click
                                if let Some(idx) = cx[s].hovered_key_idx {
                                    cx[s].keys[idx].held = Some(Instant::now());
                                    cx[s].on_note_begin.lock().unwrap().run(cx[s].keys[idx].id);
                                }
                            }
                            GestureState::Ended => {
                                cx[s].dragging = false;
                            }
                            _ => {}
                        }
                    }
                })
                .anim(move |cx, _| {
                    // Handle mouse release
                    if !cx[s].dragging {
                        let keys_to_release: Vec<usize> = cx[s]
                            .keys
                            .iter()
                            .enumerate()
                            .filter_map(
                                |(idx, key)| {
                                    if key.held.is_some() {
                                        Some(idx)
                                    } else {
                                        None
                                    }
                                },
                            )
                            .collect();

                        for idx in keys_to_release {
                            cx[s].keys[idx].held = None;
                            cx[s].on_note_end.lock().unwrap().run(cx[s].keys[idx].id);
                        }
                    }
                })
            },
        )
    }

    /// Determines if a key position corresponds to a white key
    /// White keys follow the pattern of standard piano keys (C, D, E, F, G, A, B)
    fn is_white_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
    }

    /// Determines if a key position corresponds to a black key
    /// Black keys are the sharp/flat notes (C#, D#, F#, G#, A#)
    fn is_black_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 1 | 3 | 6 | 8 | 10)
    }
}

/// Represents the state of a single key on the MIDI keyboard
#[derive(Clone, Copy, Debug)]
struct MidiKeyState {
    /// The musical note associated with this key
    id: MidiNote,
    /// Timestamp of when the key was pressed, None if not pressed
    held: Option<Instant>,
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
    /// This is necessary to update the visual state of the keyboard
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
                let note: MidiNoteType = MidiNoteType::try_from(index as u8 % 12).unwrap();
                let octave: u8 = start_octave + (index / 12) as u8;
                MidiKeyState {
                    id: MidiNote { note, octave },
                    held: None,
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
}
