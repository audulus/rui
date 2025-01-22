use core::f32;
use std::convert::TryFrom;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

use rui::*;

/// Represents a single key on the keyboard (whether it’s held or not).
#[derive(Clone, Copy, Debug)]
struct KeyState {
    id: KeyBoardKey,
    held: Option<Instant>,
}

/// Represents the state of the entire keyboard.
struct KeyBoardState {
    keys: Vec<KeyState>,
    num_keys: usize,
    num_white_keys: usize,
    hover_pos: Option<LocalPoint>,
    on_key_pressed: Arc<Mutex<WithCall>>,
    on_key_released: Arc<Mutex<WithCall>>,
}

impl KeyBoardState {
    fn new(config: &KeyBoardConfig) -> Self {
        let num_keys = config.num_keys;
        let num_white_keys = Self::calculate_white_key_count(num_keys);
        let start_octave = 4;

        let keys = (0..num_keys)
            .map(|index| {
                let note: KeyBoardNote = KeyBoardNote::try_from(index as u8 % 12).unwrap();
                let octave: u8 = start_octave + (index / 12) as u8;
                KeyState {
                    id: KeyBoardKey { note, octave },
                    held: None,
                }
            })
            .collect();

        Self {
            keys,
            num_keys,
            num_white_keys,
            hover_pos: None,
            on_key_pressed: config.on_key_pressed.clone(),
            on_key_released: config.on_key_released.clone(),
        }
    }

    fn calculate_white_key_count(num_keys: usize) -> usize {
        let total_octaves = num_keys / 12;
        let remainder = num_keys % 12;
        total_octaves * 7 + Self::white_key_count_in_remainder(remainder)
    }

    fn white_key_count_in_remainder(remainder: usize) -> usize {
        match remainder {
            0 => 0,
            1 | 3 | 4 | 5 | 7 | 8 | 10 | 11 => 1,
            _ => 2,
        }
    }
}

pub type KeyBoardNoteFreq = f32;

impl TryFrom<KeyBoardKey> for KeyBoardNoteFreq {
    type Error = ();

    fn try_from(value: KeyBoardKey) -> Result<Self, Self::Error> {
        let note_freqs = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0];

        let freq = 440.0
            * f32::powf(
                2.0,
                (value.octave as f32 - 4.0) + note_freqs[value.note as usize] / 12.0,
            );
        Ok(freq)
    }
}

pub type KeyBoardNoteU8 = u8;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum KeyBoardNote {
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

impl TryFrom<KeyBoardKey> for KeyBoardNoteU8 {
    type Error = ();

    fn try_from(value: KeyBoardKey) -> Result<Self, Self::Error> {
        let note: u8 = match value.note {
            KeyBoardNote::C => 0,
            KeyBoardNote::CSharp => 1,
            KeyBoardNote::D => 2,
            KeyBoardNote::DSharp => 3,
            KeyBoardNote::E => 4,
            KeyBoardNote::F => 5,
            KeyBoardNote::FSharp => 6,
            KeyBoardNote::G => 7,
            KeyBoardNote::GSharp => 8,
            KeyBoardNote::A => 9,
            KeyBoardNote::ASharp => 10,
            KeyBoardNote::B => 11,
        };

        let octave = value.octave as u8;
        Ok(octave * 12 + note)
    }
}

impl TryFrom<KeyBoardNoteU8> for KeyBoardNote {
    type Error = ();

    fn try_from(value: KeyBoardNoteU8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(KeyBoardNote::C),
            1 => Ok(KeyBoardNote::CSharp),
            2 => Ok(KeyBoardNote::D),
            3 => Ok(KeyBoardNote::DSharp),
            4 => Ok(KeyBoardNote::E),
            5 => Ok(KeyBoardNote::F),
            6 => Ok(KeyBoardNote::FSharp),
            7 => Ok(KeyBoardNote::G),
            8 => Ok(KeyBoardNote::GSharp),
            9 => Ok(KeyBoardNote::A),
            10 => Ok(KeyBoardNote::ASharp),
            11 => Ok(KeyBoardNote::B),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyBoardKey {
    pub note: KeyBoardNote,
    pub octave: u8,
}

struct WithCall {
    fp: Box<dyn Fn(KeyBoardKey)>,
}

impl WithCall {
    pub fn new(fp: impl Fn(KeyBoardKey) + 'static) -> Self {
        WithCall { fp: Box::new(fp) }
    }

    pub fn run(&self, key: KeyBoardKey) {
        (self.fp)(key);
    }
}

pub struct KeyBoardConfig {
    num_keys: usize,
    on_key_pressed: Arc<Mutex<WithCall>>,
    on_key_released: Arc<Mutex<WithCall>>,
}

impl KeyBoardConfig {
    pub fn new() -> Self {
        Self {
            num_keys: 25,
            on_key_pressed: Arc::new(Mutex::new(WithCall::new(|_| {}))),
            on_key_released: Arc::new(Mutex::new(WithCall::new(|_| {}))),
        }
    }

    pub fn num_keys(mut self, num_keys: usize) -> Self {
        self.num_keys = num_keys;
        self
    }

    pub fn on_key_pressed(mut self, on_key_change: impl Fn(KeyBoardKey) + 'static) -> Self {
        self.on_key_pressed = Arc::new(Mutex::new(WithCall::new(on_key_change)));
        self
    }

    pub fn on_key_released(mut self, on_key_change: impl Fn(KeyBoardKey) + 'static) -> Self {
        self.on_key_released = Arc::new(Mutex::new(WithCall::new(on_key_change)));
        self
    }

    pub fn show(self) -> impl View {
        KeyBoard::without_config().show(self)
    }
}

#[derive(Default, Clone)]
pub struct KeyBoard;

impl KeyBoard {
    pub fn new() -> KeyBoardConfig {
        KeyBoardConfig::new()
    }

    pub fn without_config() -> Self {
        Self::default()
    }

    pub fn show(self, config: KeyBoardConfig) -> impl View {
        state(
            move || KeyBoardState::new(&config),
            |s, _| {
                canvas(move |cx, rect, vger| {
                    let white_key_width = rect.width() / cx[s].num_white_keys as f32;
                    let key_height = rect.height();
                    let black_key_height = key_height * 0.6;
                    let black_key_width = white_key_width * 0.7;
                    let mut white_key_count = 0;
                    let mut hovered_key_idx: Option<usize> = None;

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
                            );
                            if let Some(hover_pos) = cx[s].hover_pos {
                                if hover_pos.x >= x && hover_pos.x <= x + white_key_width {
                                    hovered_key_idx = Some(key_pos);
                                }
                            }
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
                            );
                            if let Some(hover_pos) = cx[s].hover_pos {
                                if hover_pos.x >= x
                                    && hover_pos.x <= x + black_key_width
                                    && hover_pos.y >= (key_height - black_key_height)
                                    && hover_pos.y <= key_height
                                {
                                    hovered_key_idx = Some(key_pos);
                                }
                            }
                        }
                    }

                    // Handle mouse click for key press/release
                    if cx.mouse_buttons.left {
                        if let Some(idx) = hovered_key_idx {
                            cx[s].keys[idx].held = Some(Instant::now());
                            cx[s].on_key_pressed.lock().unwrap().run(cx[s].keys[idx].id);
                        }
                    }

                    // Release keys not hovered
                    let keys_to_release: Vec<usize> = cx[s]
                        .keys
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, key)| {
                            if key.held.is_some() && hovered_key_idx != Some(idx) {
                                Some(idx)
                            } else {
                                None
                            }
                        })
                        .collect();

                    for idx in keys_to_release {
                        cx[s].keys[idx].held = None;
                        cx[s]
                            .on_key_released
                            .lock()
                            .unwrap()
                            .run(cx[s].keys[idx].id);
                    }
                })
                .hover(move |cx, hover| {
                    if !hover {
                        cx[s].hover_pos = None;
                    }
                })
                .hover_p(move |cx, position| {
                    cx[s].hover_pos = Some(position);
                })
            },
        )
    }

    fn is_white_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 0 | 2 | 4 | 5 | 7 | 9 | 11)
    }

    fn is_black_key(key_pos: usize) -> bool {
        matches!(key_pos % 12, 1 | 3 | 6 | 8 | 10)
    }

    fn draw_white_key(vger: &mut Vger, x: f32, y: f32, width: f32, height: f32, held: bool) {
        let color = if held {
            vger::Color::new(0.8, 0.8, 0.8, 1.0)
        } else {
            vger::Color::new(0.9, 0.9, 0.9, 1.0)
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 0.0, paint_index);
    }

    fn draw_black_key(vger: &mut Vger, x: f32, y: f32, width: f32, height: f32, held: bool) {
        let base_color = vger::Color::new(0.05, 0.05, 0.05, 1.0);
        let color = if held {
            vger::Color::new(
                base_color.r + 0.05,
                base_color.g + 0.05,
                base_color.b + 0.05,
                1.0,
            )
        } else {
            base_color
        };
        let paint_index = vger.color_paint(color);
        let rect = LocalRect::new(LocalPoint::new(x, y), LocalSize::new(width, height));
        vger.fill_rect(rect, 0.0, paint_index);
    }
}
