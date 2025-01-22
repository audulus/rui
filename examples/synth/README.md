# MIDI Synthesizer

A simple MIDI synthesizer built with Rust, featuring a virtual piano keyboard interface and real-time audio synthesis.

![Synth](https://github.com/user-attachments/assets/a4506954-a459-4fe9-a0a5-f7de9ff58d44)

## Features

- Interactive virtual MIDI keyboard
- Real-time audio synthesis
- Configurable number of keys
- Support for both white and black keys
- Visual feedback for pressed keys

## Dependencies

- `rui`: For the user interface
- `rodio`: For audio synthesis and playback
- `palette`: For color management
- `enterpolation`: For value interpolation

## Running the Application

To run the synthesizer:

```shell
cargo run -p synth
```

## Usage

- Click and hold piano keys to play notes
- Release to stop the sound
- The keyboard starts in octave 4 (middle C)
- Default configuration includes 25 keys (2 octaves + 1 note)

## Development

The project consists of three main components:

1. MIDI Keyboard UI (`midi_keyboard.rs`)
2. Sound Synthesis (`synth`)
3. Main Application (`main.rs`)

# Todo

- Use view modifiers for all events.
- Add keyboard input

## License

MIT

## Acknowledgments

This calculator uses the following Rust crates:

- `rui` for the user interface
- `enterpolation` for color gradients
- `palette` for color management
