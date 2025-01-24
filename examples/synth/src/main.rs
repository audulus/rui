use rodio::source::Source;
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

mod midi_keyboard;
mod synth;

use midi_keyboard::{MidiFrequency, MidiKeyboard, MidiNoteId};
use synth::{Oscillator, Synth};

use rui::*;

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));
    let synth_clone = synth.clone();
    let synth_clone_update = synth.clone();

    std::thread::spawn(move || loop {
        synth_clone_update.lock().unwrap().update();
    });

    // Create and configure the MIDI keyboard
    MidiKeyboard::new()
        .num_keys(25)
        .start_octave(4)
        .max_simultaneous_keys(5)
        .on_note_begin(move |event| {
            let note = event.note;

            let mut synth = synth.lock().unwrap();

            // Get the frequency of the note.
            let frequency: MidiFrequency = note.frequency();

            // Create an audio source for the note.
            let audio_source = Oscillator::sine_wave(frequency).amplify(1.0);

            // Get the note id (u8) if you need it. 0 is the lowest note. 127 is the highest note.
            let source_id: MidiNoteId = note.id();

            // Send the audio source to the synth.
            synth.play_source(Box::new(audio_source), source_id);
        })
        .on_note_end(move |event| {
            let note = event.note;

            let mut synth = synth_clone.lock().unwrap();
            let source_id: MidiNoteId = note.id();
            synth.release_source(source_id);
        })
        .show()
        .size([400.0, 200.0])
        .run();
}
