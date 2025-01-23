//! A calculator implementation with a modern UI, supporting both dark and light modes.
//! This calculator provides basic arithmetic operations, keyboard input support,
//! and a responsive touch-friendly interface.

use enterpolation::{linear::ConstEquidistantLinear, Generator};
use palette::LinSrgb;
use rui::*;

/// Represents the state of the clear button functionality.
/// Used to toggle between Clear (C) and All Clear (AC) modes.
#[derive(PartialEq, Clone, Copy)]
enum ClearState {
    /// Initial state - displays "C"
    Initial,
    /// State after clearing - displays "AC"
    JustCleared,
}

/// Represents the basic arithmetic operations supported by the calculator.
#[derive(Clone, Copy)]
enum Operator {
    Add,      // Addition (+)
    Subtract, // Subtraction (-)
    Multiply, // Multiplication (*)
    Divide,   // Division (/)
}

/// Represents special calculator operations beyond basic arithmetic.
#[derive(Clone, Copy)]
enum SpecialOperator {
    Clear,      // Clear current value or all values (C/AC)
    ToggleSign, // Toggle between positive/negative (+/-)
    Percentage, // Convert to percentage (%)
    Decimal,    // Add decimal point (.)
    Equals,     // Calculate result (=)
}

/// Represents a calculator button's type and value.
#[derive(Clone)]
enum Button {
    /// Numeric digit buttons (0-9)
    Digit(u64),
    /// Arithmetic operator buttons (+, -, *, /)
    Operator(Operator),
    /// Special function buttons (Clear, +/-, %, ., =)
    Special(SpecialOperator),
}

/// Tracks the interactive state of a calculator button.
struct ButtonState {
    /// Whether the mouse is currently hovering over the button
    is_hovered: bool,
    /// Whether the button is currently being pressed
    is_touched: bool,
}

/// Configuration options for the calculator's appearance.
#[derive(Clone)]
pub struct CalculatorConfig {
    /// Whether to use dark mode colors
    dark_mode: bool,
    /// Whether to use rounded corners on the calculator frame
    rounded_corners: bool,
}

impl CalculatorConfig {
    /// Enable dark mode for the calculator.
    /// Changes the color scheme to use darker backgrounds and lighter text.
    pub fn dark_mode(mut self) -> Self {
        self.dark_mode = true;
        self
    }

    /// Enable rounded corners for the calculator frame.
    /// Particularly useful when the calculator is displayed as a widget.
    pub fn rounded_corners(mut self) -> Self {
        self.rounded_corners = true;
        self
    }

    /// Create and display the calculator with the current configuration.
    pub fn show(self) -> impl View {
        Calculator::with_config(&self).show()
    }
}

/// The main calculator structure handling rendering and state management.
#[derive(Clone)]
pub struct Calculator {
    /// Color gradient for number buttons
    ocean: enterpolation::linear::Linear<
        enterpolation::ConstEquidistant<f32, 2>,
        [palette::rgb::Rgb<palette::encoding::Linear<palette::encoding::Srgb>>; 2],
        enterpolation::Identity,
    >,
    /// Color gradient for operator buttons
    sky: enterpolation::linear::Linear<
        enterpolation::ConstEquidistant<f32, 2>,
        [palette::rgb::Rgb<palette::encoding::Linear<palette::encoding::Srgb>>; 2],
        enterpolation::Identity,
    >,
    /// Text color based on theme
    text_color: vger::Color,
    /// Background color based on theme
    background_color: vger::Color,
    /// Corner radius for the calculator frame
    background_corner_radius: f32,
    /// Color for the number display area
    number_display_color: vger::Color,
}

impl Calculator {
    pub fn new() -> CalculatorConfig {
        CalculatorConfig {
            dark_mode: false,
            rounded_corners: false,
        }
    }

    pub fn with_config(config: &CalculatorConfig) -> Calculator {
        let ocean_colors = if config.dark_mode {
            [
                LinSrgb::new(0.05, 0.15, 0.30),
                LinSrgb::new(0.1, 0.20, 0.25),
            ]
        } else {
            [
                LinSrgb::new(0.30, 0.55, 0.65),
                LinSrgb::new(0.45, 0.80, 0.70),
            ]
        };

        let ocean = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked(ocean_colors);

        let sky_colors = if config.dark_mode {
            [
                LinSrgb::new(0.00, 0.10, 0.30),
                LinSrgb::new(0.05, 0.15, 0.35),
            ]
        } else {
            [LinSrgb::new(0.3, 0.5, 0.80), LinSrgb::new(0.3, 0.4, 0.90)]
        };

        let sky = ConstEquidistantLinear::<f32, _, 2>::equidistant_unchecked(sky_colors);

        let text_color = if config.dark_mode {
            vger::Color::new(1.0, 1.0, 1.0, 1.0)
        } else {
            vger::Color::new(0.0, 0.0, 0.0, 1.0)
        };

        let background_color = if config.dark_mode {
            vger::Color::new(0.05, 0.05, 0.05, 1.0)
        } else {
            vger::Color::new(1.0, 1.0, 1.0, 1.0)
        };

        let number_display_color = if config.dark_mode {
            vger::Color::new(0.1, 0.1, 0.1, 1.0)
        } else {
            vger::Color::new(0.8, 0.8, 0.8, 1.0)
        };

        let background_corner_radius = if config.rounded_corners { 15.0 } else { 0.0 };

        Calculator {
            ocean,
            sky,
            text_color,
            background_color,
            background_corner_radius,
            number_display_color,
        }
    }

    fn show(self) -> impl View {
        focus(move |has_focus| {
            let calculator = self.clone();

            state(
                move || CalculatorState::new(),
                move |s: StateHandle<CalculatorState>, cx: &Context| {
                    zstack((
                        rectangle()
                            .color(self.background_color)
                            .corner_radius(self.background_corner_radius)
                            .key(move |cx, k| {
                                if has_focus {
                                    cx[s].key(&k);
                                }
                            }),
                        vstack((
                            calculator.display_value(&cx[s]),
                            hstack((
                                calculator.button_view(
                                    s,
                                    Button::Special(SpecialOperator::Clear),
                                    0,
                                ),
                                calculator.button_view(
                                    s,
                                    Button::Special(SpecialOperator::ToggleSign),
                                    1,
                                ),
                                calculator.button_view(
                                    s,
                                    Button::Special(SpecialOperator::Percentage),
                                    2,
                                ),
                                calculator.button_view(s, Button::Operator(Operator::Divide), 3),
                            )),
                            hstack((
                                calculator.button_view(s, Button::Digit(7), 4),
                                calculator.button_view(s, Button::Digit(8), 5),
                                calculator.button_view(s, Button::Digit(9), 6),
                                calculator.button_view(s, Button::Operator(Operator::Multiply), 7),
                            )),
                            hstack((
                                calculator.button_view(s, Button::Digit(4), 8),
                                calculator.button_view(s, Button::Digit(5), 9),
                                calculator.button_view(s, Button::Digit(6), 10),
                                calculator.button_view(s, Button::Operator(Operator::Subtract), 11),
                            )),
                            hstack((
                                calculator.button_view(s, Button::Digit(1), 12),
                                calculator.button_view(s, Button::Digit(2), 13),
                                calculator.button_view(s, Button::Digit(3), 14),
                                calculator.button_view(s, Button::Operator(Operator::Add), 15),
                            )),
                            hstack((
                                calculator.button_view(s, Button::Digit(0), 16),
                                calculator.button_view(
                                    s,
                                    Button::Special(SpecialOperator::Decimal),
                                    17,
                                ),
                                calculator.button_view(
                                    s,
                                    Button::Special(SpecialOperator::Equals),
                                    18,
                                ),
                            )),
                        ))
                        .padding(Auto),
                    ))
                },
            )
        })
    }

    fn display_value(&self, state: &CalculatorState) -> impl View {
        let display_text = if state.has_error {
            "Error".to_string()
        } else if state.second_operand.is_empty() {
            "0".to_string()
        } else {
            let mut string = state.second_operand.clone();
            if string.len() > 10 {
                string.truncate(10);
                // add an ellipsis to indicate that the number is too long.
                string.push_str("…");
                string
            } else {
                string
            }
        };
        zstack((
            rectangle()
                .corner_radius(10.0)
                .color(self.number_display_color),
            text(&display_text)
                .color(self.text_color)
                .font_size(40)
                .size([0.0, 50.0])
                .offset([10.0, 10.0]),
        ))
        .padding(Auto)

        // canvas(move |_, rect, vger| {
        //     vger.save();
        //     let color = vger::Color::new(0.2, 0.2, 0.2, 1.0);
        //     let paint_index = vger.color_paint(color);
        //     vger.fill_rect(rect, 10.0, paint_index);

        //     vger.restore();
        //     vger.save();

        //     let text_height: u32 = 40;

        //     let origin = vger.text_bounds(&display_text, text_height, None).origin;

        //     vger.translate([
        //         -origin.x,
        //         -origin.y + rect.height() / 2.0 - (text_height as f32) / 2.0,
        //     ]);

        //     let text_color = vger::Color::new(1.0, 1.0, 1.0, 1.0);

        //     vger.text(&display_text, text_height, text_color, Some(0.0));
        //     vger.restore();
        // })
    }

    fn button_view(&self, s: StateHandle<CalculatorState>, button: Button, id: usize) -> impl View {
        let calculator = self.clone();

        let is_number = matches!(button, Button::Digit(_));
        let digit = match button {
            Button::Digit(d) => d,
            _ => 0,
        };

        state(
            || ButtonState {
                is_hovered: false,
                is_touched: false,
            },
            move |button_state, cx: &Context| {
                let button_clone = button.clone();

                let alpha = if cx[button_state].is_touched {
                    0.5
                } else if cx[button_state].is_hovered {
                    0.8
                } else {
                    1.0
                };
                let color = {
                    let color = if is_number {
                        // cx[s].blue_gradient.gen(id as f32 / 18.0)
                        calculator.ocean.gen(digit as f32 / 9.0)
                    } else {
                        calculator.sky.gen(id as f32 / 18.0)
                    };
                    vger::Color::new(color.red, color.green, color.blue, alpha)
                };

                zstack((
                    rectangle()
                        .corner_radius(10.0)
                        .color(color)
                        .touch(move |cx, info| match info.state {
                            TouchState::Begin => {
                                cx[button_state].is_touched = true;
                            }
                            TouchState::End => {
                                cx[button_state].is_touched = false;
                                cx[s].button_action(button_clone.clone());
                            }
                        })
                        .hover(move |cx, hovered| {
                            cx[button_state].is_hovered = hovered;
                        }),
                    {
                        let label = match button {
                            Button::Digit(digit) => digit.to_string(),
                            Button::Operator(op) => match op {
                                Operator::Add => "+".to_string(),
                                Operator::Subtract => "-".to_string(),
                                Operator::Multiply => "*".to_string(),
                                Operator::Divide => "/".to_string(),
                            },
                            Button::Special(action) => match action {
                                SpecialOperator::Clear => {
                                    if cx[s].clear_state == ClearState::JustCleared {
                                        "AC".to_string()
                                    } else {
                                        "C".to_string()
                                    }
                                }
                                SpecialOperator::ToggleSign => "+/-".to_string(),
                                SpecialOperator::Percentage => "%".to_string(),
                                SpecialOperator::Equals => "=".to_string(),
                                SpecialOperator::Decimal => ".".to_string(),
                            },
                        };

                        text(&label)
                    }
                    .font_size(30)
                    .color(calculator.text_color)
                    .offset([10.0, 10.0]),
                ))
                .padding(Auto)
            },
        )
    }
}

/// Represents the current state of the calculator's computation.
pub struct CalculatorState {
    /// The first operand in the current calculation
    first_operand: String,
    /// The second operand or current input
    second_operand: String,
    /// The currently selected arithmetic operator
    current_operator: Option<Operator>,
    /// Whether the next input should start a new number
    is_input_new: bool,
    /// Whether a calculation error has occurred (e.g., division by zero)
    has_error: bool,
    /// Whether the current display shows a calculation result
    is_result_displayed: bool,
    /// The last operator used (for repeat calculations)
    last_operator: Option<Operator>,
    /// Current state of the clear button (C/AC)
    clear_state: ClearState,
}

impl CalculatorState {
    fn new() -> Self {
        Self {
            first_operand: "0".to_string(),
            second_operand: "0".to_string(),
            current_operator: None,
            is_input_new: true,
            has_error: false,
            is_result_displayed: false,
            last_operator: None,
            clear_state: ClearState::Initial,
        }
    }

    /// Executes the current arithmetic operation.
    /// Handles basic error cases like division by zero.
    fn execute_operation(&mut self) {
        if self.has_error {
            return;
        }

        let first_operand: f64 = self.first_operand.parse().unwrap_or(0.0);
        let second_operand: f64 = self.second_operand.parse().unwrap_or(0.0);

        let result = match self.current_operator {
            Some(Operator::Add) => first_operand + second_operand,
            Some(Operator::Subtract) => first_operand - second_operand,
            Some(Operator::Multiply) => first_operand * second_operand,
            Some(Operator::Divide) => {
                if second_operand == 0.0 {
                    self.has_error = true;
                    return; // Early return on division by zero
                }
                first_operand / second_operand
            }
            None => return, // Handle the case where no operator is set
        };

        self.second_operand = result.to_string();
        self.first_operand = self.second_operand.clone();
        self.is_input_new = true;
        self.is_result_displayed = true;
        self.current_operator = None;
    }

    /// Handles digit input, managing leading zeros and decimal points.
    fn input_digit(&mut self, digit: u64) {
        if self.is_result_displayed {
            self.second_operand = String::new(); // Clear on new input after result
            self.is_result_displayed = false;
        }

        if self.second_operand == "0" && digit == 0 && !self.second_operand.contains('.') {
            return; // Prevent multiple leading zeros before decimal
        }
        if self.second_operand == "0" && !self.second_operand.contains('.') {
            self.second_operand = String::new();
        }

        self.second_operand.push_str(&digit.to_string());
    }

    /// Processes decimal point input, ensuring only one decimal point exists.
    fn input_decimal(&mut self) {
        if self.is_result_displayed {
            self.is_result_displayed = false; // Important: Clear the flag *before* modifying the operand
        }

        if !self.second_operand.contains('.') {
            if self.second_operand.is_empty() {
                self.second_operand.push_str("0.");
            } else {
                self.second_operand.push('.');
            }
        }
    }

    /// Toggles the sign of the current number between positive and negative.
    fn toggle_sign(&mut self) {
        if !self.second_operand.is_empty() && self.second_operand != "0" {
            if self.second_operand.starts_with('-') {
                self.second_operand = self.second_operand[1..].to_string();
            } else {
                self.second_operand = format!("-{}", self.second_operand);
            }
        }
    }

    /// Converts the current number to a percentage (divides by 100).
    fn apply_percentage(&mut self) {
        if let Ok(value) = self.second_operand.parse::<f64>() {
            self.second_operand = (value / 100.0).to_string();
        }
    }

    /// Resets the calculator to its initial state.
    fn reset(&mut self) {
        self.first_operand = "0".to_string();
        self.second_operand = "0".to_string();
        self.current_operator = None;
        self.last_operator = None;
        self.is_input_new = true;
        self.has_error = false;
        self.is_result_displayed = false;
        self.clear_state = ClearState::Initial;
    }

    /// Processes button presses and updates calculator state accordingly.
    fn button_action(&mut self, button: Button) {
        match button {
            Button::Digit(_)
            | Button::Operator(_)
            | Button::Special(SpecialOperator::ToggleSign)
            | Button::Special(SpecialOperator::Percentage)
            | Button::Special(SpecialOperator::Equals)
            | Button::Special(SpecialOperator::Decimal) => {
                if self.clear_state == ClearState::JustCleared {
                    self.reset();
                } else {
                    self.clear_state = ClearState::Initial;
                }

                match button {
                    Button::Digit(digit) => self.input_digit(digit),
                    Button::Operator(op) => {
                        if !self.second_operand.is_empty() {
                            if self.current_operator.is_some() {
                                self.execute_operation();
                            } else {
                                self.first_operand = self.second_operand.clone();
                                self.second_operand = "0".to_string();
                            }
                        }
                        self.current_operator = Some(op);
                        self.is_input_new = true;
                        self.is_result_displayed = false;
                    }
                    Button::Special(action) => match action {
                        SpecialOperator::ToggleSign => self.toggle_sign(),
                        SpecialOperator::Percentage => self.apply_percentage(),
                        SpecialOperator::Equals => {
                            if self.current_operator.is_some() && !self.second_operand.is_empty() {
                                self.execute_operation();
                            }
                        }
                        SpecialOperator::Decimal => self.input_decimal(),
                        _ => (),
                    },
                }
            }
            Button::Special(SpecialOperator::Clear) => {
                if self.second_operand == "0"
                    && self.first_operand == "0"
                    && self.current_operator.is_none()
                {
                    self.reset();
                    self.clear_state = ClearState::JustCleared;
                } else {
                    self.second_operand = "0".to_string();
                    self.is_input_new = true;
                    self.clear_state = ClearState::Initial;
                }
            }
        }
    }

    /// Handles keyboard input, mapping keys to calculator functions.
    fn key(&mut self, k: &Key) {
        match k {
            Key::Backspace => {
                if self.second_operand.len() > 0 {
                    self.second_operand.pop();
                }
            }
            Key::Space => {
                // clear input
                self.reset();
            }
            Key::Enter => {
                self.button_action(Button::Special(SpecialOperator::Equals));
            }
            Key::Character(c) => {
                if c.is_ascii_digit() {
                    self.input_digit(c.to_digit(10).unwrap() as u64);
                } else {
                    match c {
                        'c' | 'C' => self.button_action(Button::Special(SpecialOperator::Clear)),
                        '+' => self.button_action(Button::Operator(Operator::Add)),
                        '-' => self.button_action(Button::Operator(Operator::Subtract)),
                        '*' => self.button_action(Button::Operator(Operator::Multiply)),
                        '/' => self.button_action(Button::Operator(Operator::Divide)),
                        '%' => self.button_action(Button::Special(SpecialOperator::Percentage)),
                        '.' => self.button_action(Button::Special(SpecialOperator::Decimal)),
                        '=' => self.button_action(Button::Special(SpecialOperator::Equals)),
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }
}
