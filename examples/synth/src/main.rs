// use midir::MidiInput;
// use rodio::source::Source;
// use rodio::OutputStream;
// use std::sync::{Arc, Mutex};

mod keyboard;
mod oscillator;
mod synth;

use keyboard::KeyBoard;
// use oscillator::Oscillator;
// use synth::Synth;

use rui::*;

fn main() {
    // Create a new synth
    // let (_stream, stream_handle) = OutputStream::try_default().unwrap(); // Oddly, this cant be done in the new function, otherwise the program will panic
    // The synth will manage multiple audio sinks and their envelopes
    // let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));

    // // Create a new midi input
    // let midi_in = MidiInput::new("midir reading input").unwrap();

    // // Get an input port (Automatically choosing the first one)
    // // (It will panic if no midi device is connected)
    // let in_port = &midi_in.ports()[0];

    // // Cloned for use in closure, because the closure takes ownership
    // // However this is not a deep clone, so the audio sinks and their envelopes stay the same!
    // let synth_clone = synth.clone();

    // // _conn_in needs to be a named parameter, because it needs to be kept alive until the end of the scope
    // let _conn_in = midi_in
    //     .connect(
    //         in_port,
    //         "midir-read-input",
    //         move |_stamp, message, _| {
    //             // Message is in the format of [event, key, pressure]
    //             let hz = 440.0 * 2.0_f32.powf((message[1] as f32 - 69.0) / 12.0);
    //             let pressure = message[2] as f32 / 127.0;

    //             let mut synth = synth_clone.lock().unwrap();

    //             if message[0] == 144 {
    //                 // 144 is the event for note on
    //                 // Create a new audio source from the oscillator
    //                 let audio_source = Oscillator::square_wave(hz).amplify(pressure);
    //                 // Play the audio source
    //                 synth.play_source(Box::new(audio_source), message[1]);
    //             }
    //             if message[0] == 128 {
    //                 // 128 is the event for note off
    //                 // Signals the envelope to start releasing
    //                 // The sink is automatically deleted when the envelope is done releasing
    //                 synth.release_source(message[1]);
    //             }
    //         },
    //         (),
    //     )
    //     .unwrap();

    // loop {
    //     let mut synth = synth.lock().unwrap();
    //     synth.update();
    // }

    KeyBoard::new()
        .num_keys(25)
        .show()
        .size([400.0, 200.0])
        .run();
}
