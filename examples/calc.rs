use rui::*;

// Struct to represent the calculator's state.
struct CalculatorState {
    first_operand: String,              // Stores the first operand for operations.
    second_operand: String,             // Stores the second operand or result after "=".
    current_operator: Option<Operator>, // Stores the currently selected operator.
    is_input_new: bool,                 // Flag to check if the input is new.
    has_error: bool,                    // Indicates if there's an error (e.g., division by zero).
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self {
            first_operand: "0".to_string(),
            second_operand: "0".to_string(),
            current_operator: None,
            is_input_new: true,
            has_error: false,
        }
    }
}

// Enum to represent arithmetic operations.
enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl CalculatorState {
    // Perform the pending operation based on the current operator.
    fn execute_operation(&mut self) {
        if self.has_error {
            return; // Skip operation if there’s an error.
        }

        // Convert operands to f64 and handle any parsing failures gracefully.
        let first_operand: f64 = self.first_operand.parse().unwrap_or(0.0);
        let second_operand: f64 = self.second_operand.parse().unwrap_or(0.0);

        match &self.current_operator {
            Some(Operator::Add) => {
                self.second_operand = (first_operand + second_operand).to_string()
            }
            Some(Operator::Subtract) => {
                self.second_operand = (first_operand - second_operand).to_string()
            }
            Some(Operator::Multiply) => {
                self.second_operand = (first_operand * second_operand).to_string()
            }
            Some(Operator::Divide) => {
                if second_operand != 0.0 {
                    self.second_operand = (first_operand / second_operand).to_string();
                } else {
                    self.has_error = true; // Set error state on division by zero.
                    return;
                }
            }
            _ => {}
        }

        // Set the first operand to the result for subsequent operations.
        self.first_operand = self.second_operand.clone();
        self.is_input_new = true;
    }

    // Handle digit input.
    fn input_digit(&mut self, digit: u64) {
        if self.is_input_new {
            self.second_operand.clear(); // Start fresh input.
            self.is_input_new = false;
        }
        self.second_operand.push_str(&digit.to_string());
    }

    // Handle decimal point input.
    fn input_decimal(&mut self) {
        if self.is_input_new {
            self.second_operand.clear(); // Start fresh input.
            self.is_input_new = false;
        }
        if !self.second_operand.contains('.') {
            self.second_operand.push('.');
        }
    }

    // Toggle the sign of the current input (positive/negative).
    fn toggle_sign(&mut self) {
        if !self.second_operand.is_empty() && self.second_operand != "0" {
            if self.second_operand.starts_with('-') {
                self.second_operand = self.second_operand[1..].to_string();
            } else {
                self.second_operand = format!("-{}", self.second_operand);
            }
        }
    }

    // Apply percentage to the current input.
    fn apply_percentage(&mut self) {
        if let Ok(value) = self.second_operand.parse::<f64>() {
            self.second_operand = (value / 100.0).to_string();
        }
    }

    // Reset the calculator to its initial state.
    fn reset(&mut self) {
        self.first_operand.clear();
        self.second_operand.clear();
        self.current_operator = None;
        self.is_input_new = true;
        self.has_error = false;
    }
}

// Helper function to create digit buttons.
fn digit_button(digit: u64, state: StateHandle<CalculatorState>) -> impl View {
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(RED_HIGHLIGHT_BACKGROUND)
            .tap(move |cx| {
                cx[state].input_digit(digit);
            }),
        text(&digit.to_string())
            .color(TEXT_COLOR)
            .offset([10.0, 10.0]),
    ))
    .padding(Auto)
}

// Helper function to create operation buttons.
fn operation_button(title: &str, state: StateHandle<CalculatorState>) -> impl View {
    let title: String = title.to_string();
    let title_copy = title.to_string();
    zstack((
        rectangle()
            .corner_radius(10.0)
            .color(AZURE_HIGHLIGHT_BACKGROUND)
            .tap(move |cx| {
                match title.as_str() {
                    "AC" => {
                        cx[state].reset();
                    }
                    "+/-" => {
                        cx[state].toggle_sign();
                    }
                    "%" => {
                        cx[state].apply_percentage();
                    }
                    "=" => {
                        cx[state].execute_operation();
                        cx[state].current_operator = None;
                    }
                    "/" => {
                        cx[state].execute_operation();
                        cx[state].current_operator = Some(Operator::Divide);
                        cx[state].second_operand.clear(); // Clear current value.
                    }
                    "*" => {
                        cx[state].execute_operation();
                        cx[state].current_operator = Some(Operator::Multiply);
                        cx[state].second_operand.clear(); // Clear current value.
                    }
                    "-" => {
                        cx[state].execute_operation();
                        cx[state].current_operator = Some(Operator::Subtract);
                        cx[state].second_operand.clear(); // Clear current value.
                    }
                    "+" => {
                        cx[state].execute_operation();
                        cx[state].current_operator = Some(Operator::Add);
                        cx[state].second_operand.clear(); // Clear current value.
                    }
                    "." => {
                        cx[state].input_decimal(); // Handle decimal input here.
                    }
                    _ => {}
                }
            }),
        text(&title_copy).offset([10.0, 10.0]),
    ))
    .padding(Auto)
}

// Function to display the current value or error message.
fn display_value(state: &CalculatorState) -> impl View {
    if state.has_error {
        text("Error").color(RED_HIGHLIGHT)
    } else {
        if state.second_operand.is_empty() {
            text("0")
        } else {
            text(&state.second_operand)
        }
    }
    .font_size(32)
    .size([0.0, 50.0]) // Stop the text from expanding / contracting like when its only "."
}

fn main() {
    state(
        || CalculatorState::default(),
        |s: StateHandle<CalculatorState>, cx: &Context| {
            // Calculator UI layout.
            vstack((
                display_value(&cx[s]),
                hstack((
                    operation_button("AC", s),
                    operation_button("+/-", s),
                    operation_button("%", s),
                    operation_button("/", s),
                )),
                hstack((
                    digit_button(7, s),
                    digit_button(8, s),
                    digit_button(9, s),
                    operation_button("*", s),
                )),
                hstack((
                    digit_button(4, s),
                    digit_button(5, s),
                    digit_button(6, s),
                    operation_button("-", s),
                )),
                hstack((
                    digit_button(1, s),
                    digit_button(2, s),
                    digit_button(3, s),
                    operation_button("+", s),
                )),
                hstack((
                    digit_button(0, s),
                    operation_button(".", s), // Decimal handling.
                    operation_button("=", s),
                )),
            ))
            .padding(Auto)
        },
    )
    .run()
}
