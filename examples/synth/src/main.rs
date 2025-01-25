use std::sync::{Arc, Mutex};
use std::time::Duration;

use rodio::OutputStream;

mod midi_keyboard;
mod oscillator;
mod synth;

use midi_keyboard::{MidiFrequency, MidiKeyboard, MidiNoteId};
use oscillator::Oscillator;
use synth::{AdvancedSynth, EnvelopeConfig};

use rui::Run;

fn main() {
    // Create the audio output stream
    let (_stream, stream_handle) =
        OutputStream::try_default().expect("Failed to create output stream");

    // Create a custom envelope configuration
    let custom_envelope = EnvelopeConfig::builder()
        .attack(Duration::from_millis(10))
        .decay(Duration::from_millis(50))
        .sustain(0.6)
        .release(Duration::from_millis(100))
        .build()
        .expect("Failed to create envelope configuration");

    // Initialize the advanced synthesizer with custom default envelope
    let synth = Arc::new(Mutex::new(AdvancedSynth::new(
        stream_handle,
        Some(custom_envelope),
    )));

    // Clone synthesizer references for different threads
    let synth_update = synth.clone();
    let synth_note_begin = synth.clone();
    let synth_note_end = synth.clone();

    // Spawn a dedicated thread for continuous synth updates
    std::thread::spawn(move || {
        loop {
            synth_update.lock().unwrap().update();
            std::thread::sleep(Duration::from_millis(10)); // Prevent tight spinning
        }
    });

    // Configure MIDI keyboard
    MidiKeyboard::new()
        .num_keys(25)
        .start_octave(4)
        .max_simultaneous_keys(5)
        .on_note_begin(move |event| {
            let note = event.note;
            let mut synth = synth_note_begin.lock().unwrap();

            // Get the frequency of the pressed note
            let frequency: MidiFrequency = note.frequency();

            // Create an audio source with a sawtooth wave
            let audio_source = Oscillator::square(frequency);

            // Get the note ID
            let source_id: MidiNoteId = note.id();

            // Play the source with optional custom envelope
            if let Err(e) = synth.play_source(
                Box::new(audio_source),
                source_id,
                None, // Use default envelope
            ) {
                eprintln!("Failed to play source: {}", e);
            }
        })
        .on_note_end(move |event| {
            let note = event.note;
            let mut synth = synth_note_end.lock().unwrap();
            let source_id: MidiNoteId = note.id();

            if let Err(e) = synth.release_source(source_id) {
                eprintln!("Failed to release source: {}", e);
            }
        })
        .show()
        .run();
}
