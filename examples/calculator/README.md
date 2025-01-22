# Rust Calculator

A modern, responsive calculator implementation in Rust with a clean user interface. This calculator supports both light and dark modes, keyboard input, and touch interactions.

![Calculator Light Mode](https://github.com/user-attachments/assets/fbdcde38-75cb-4152-8b76-8f07e42e8785)
![Calculator Light Mode Fullscreen](https://github.com/user-attachments/assets/a348601a-3360-49d1-a918-3455b4958d9f)
![Calculator Dark Mode Fullscreen](https://github.com/user-attachments/assets/38fe5d21-2eb2-4b9c-bcda-509743b7859d)
![Calculator Dark Mode](https://github.com/user-attachments/assets/3231c0b7-f2f5-452c-86e2-0a16fe63ba35)

## Features

### Basic Operations

- Addition (+)
- Subtraction (-)
- Multiplication (\*)
- Division (/)
- Percentage calculations (%)
- Sign toggling (+/-)
- Decimal point support

### User Interface

- Clean, modern design
- Responsive layout
- Support for both light and dark modes
- Optional rounded corners
- Error handling with clear display
- Dynamic clear button (C/AC)
- Number truncation for large values

### Input Methods

- Touch/mouse interaction
- Full keyboard support
- Automatic handling of leading zeros
- Decimal point validation

## Getting Started

### Running the Calculator

To run the calculator, use:

```shell
cargo run -p calculator
```

### Basic Usage

Create a new instance with default settings:

```rust
use calculator::Calculator;
use rui::*;

fn main() {
    Calculator::new()
        .show()
        .run();
}
```

### Customization Options

#### Dark Mode

Enable dark mode for a darker color scheme:

```rust
Calculator::new()
    .dark_mode()
    .show()
    .run();
```

#### Rounded Corners

Add rounded corners to the calculator frame:

```rust
Calculator::new()
    .rounded_corners()
    .show()
    .run();
```

#### Combine Options

You can combine multiple options:

```rust
Calculator::new()
    .dark_mode()
    .rounded_corners()
    .show()
    .run();
```

## Keyboard Controls

The calculator supports full keyboard input:

| Key       | Action            |
| --------- | ----------------- |
| 0-9       | Input digits      |
| .         | Decimal point     |
| + - \* /  | Basic operations  |
| =         | Calculate result  |
| Enter     | Calculate result  |
| %         | Percentage        |
| c or C    | Clear/All Clear   |
| Backspace | Delete last digit |
| Space     | Reset calculator  |

## Features in Detail

### Display

- Shows up to 10 digits with ellipsis (...) for longer numbers
- Clear error indication for invalid operations
- Automatic clearing after result display when starting new calculations

### Clear Button (C/AC)

- Functions as "Clear" (C) for current input
- Changes to "All Clear" (AC) when pressed twice
- Resets to initial state when AC is pressed

### Number Input

- Prevents multiple leading zeros
- Handles decimal point input intelligently
- Supports negative numbers via +/- toggle

### Operations

- Chain multiple operations
- Automatic calculation when pressing equals or new operator
- Percentage calculations
- Division by zero error handling

### Error Handling

- Displays "Error" for invalid operations
- Prevents further calculations until cleared
- Graceful handling of oversized numbers

## Development

### Project Structure

The calculator is implemented with the following key components:

- `CalculatorConfig`: Handles appearance configuration
- `Calculator`: Main implementation of the calculator UI
- `CalculatorState`: Manages calculation state and logic
- `Button`: Represents different types of calculator buttons
- Custom color gradients for visual appeal

### Building

Build the project using:

```shell
cargo build -p calculator
```

For release mode:

```shell
cargo build -p calculator --release
```

## Contributing

Contributions are welcome! Some areas for potential improvements:

- Additional mathematical functions
- Memory operations
- History tracking
- Custom themes
- Scientific notation support
- Customizable key bindings

## License

MIT

## Acknowledgments

This calculator uses the following Rust crates:

- `rui` for the user interface
- `enterpolation` for color gradients
- `palette` for color management
