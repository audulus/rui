use core::f32;

use rui::*;

// Represents the state of a single key (whether it's held or not).
#[derive(Clone, Copy)]
struct KeyState {
    held: bool,
}

impl KeyState {
    fn new() -> Self {
        Self { held: false }
    }
}

// Represents the state of the entire keyboard.
struct KeyBoardState {
    keys: Vec<KeyState>,
    num_keys: usize,
    num_white_keys: usize,
    hover_pos: Option<LocalPoint>,
}

impl KeyBoardState {
    fn new(config: &KeyBoardConfig) -> Self {
        let num_keys = config.num_keys;
        let num_white_keys = Self::calculate_white_key_count(num_keys);
        let keys = vec![KeyState::new(); num_keys];
        Self {
            keys,
            num_keys,
            num_white_keys,
            hover_pos: None,
        }
    }

    fn calculate_white_key_count(num_keys: usize) -> usize {
        let total_octaves = num_keys / 12;
        let remainder = num_keys % 12;
        let num_white_keys = total_octaves * 7 + Self::white_key_count_in_remainder(remainder);
        num_white_keys
    }

    fn white_key_count_in_remainder(remainder: usize) -> usize {
        match remainder {
            0 => 0,
            1 | 3 | 4 | 5 | 7 | 8 | 10 | 11 => 1,
            _ => 2,
        }
    }
}

pub struct KeyBoardConfig {
    pub num_keys: usize,
}

impl KeyBoardConfig {
    pub fn new() -> Self {
        Self { num_keys: 25 }
    }

    pub fn num_keys(mut self, num_keys: usize) -> Self {
        self.num_keys = num_keys;
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
            move |s, _| {
                canvas(move |cx, rect, vger| {
                    let white_key_width = rect.width() / cx[s].num_white_keys as f32;
                    let key_height = rect.height();
                    let black_key_height = key_height * 0.6;
                    let black_key_width = white_key_width * 0.7;
                    let mut white_key_count = 0;

                    // Track the hovered key index
                    let mut hovered_key_idx: Option<usize> = None;

                    // First pass: Draw all white keys
                    for key_pos in 0..cx[s].num_keys {
                        if Self::is_white_key(key_pos) {
                            let x = white_key_count as f32 * white_key_width;
                            Self::draw_white_key(
                                vger,
                                x,
                                0.0,
                                white_key_width,
                                key_height,
                                cx[s].keys[key_pos].held,
                            );

                            // Check if the current x position collides with the hover position
                            if let Some(hover_pos) = cx[s].hover_pos {
                                let hover_x = hover_pos.x;
                                if hover_x >= x && hover_x <= x + white_key_width {
                                    hovered_key_idx = Some(key_pos);
                                }
                            }

                            white_key_count += 1;
                        }
                    }

                    // Second pass: Draw all black keys
                    for key_pos in 0..cx[s].num_keys {
                        if Self::is_black_key(key_pos) {
                            let offset = match key_pos % 12 {
                                1 => 1.0,  // C#
                                3 => 2.0,  // D#
                                6 => 4.0,  // F#
                                8 => 5.0,  // G#
                                10 => 6.0, // A#
                                _ => continue,
                            };
                            let octave = (key_pos / 12) as f32;
                            let x =
                                (octave * 7.0 + offset) * white_key_width - (black_key_width / 2.0);

                            Self::draw_black_key(
                                vger,
                                x,
                                key_height - black_key_height, // Start y-coordinate adjusted for black key height
                                black_key_width,
                                black_key_height,
                                cx[s].keys[key_pos].held,
                            );

                            // Adjust collision detection for black keys: account for their shorter height
                            if let Some(hover_pos) = cx[s].hover_pos {
                                let hover_x = hover_pos.x;
                                let hover_y = hover_pos.y;

                                // Check if the hover X is within the black key's bounds
                                if hover_x >= x && hover_x <= x + black_key_width {
                                    // Adjust the Y-range check to ensure the hover is within the black key's height
                                    if hover_y >= (key_height - black_key_height)
                                        && hover_y <= key_height
                                    {
                                        hovered_key_idx = Some(key_pos);
                                    }
                                }
                            }
                        }
                    }

                    // Print the hovered key index (if any)
                    if let Some(idx) = hovered_key_idx {
                        vger.text(
                            format!("Hovered key index: {}", idx).as_str(),
                            20,
                            vger::Color::WHITE,
                            None,
                        );
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
