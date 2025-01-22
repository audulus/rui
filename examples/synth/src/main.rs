// use midir::MidiInput;
use rodio::source::Source;
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

mod keyboard;
mod oscillator;
mod synth;

use keyboard::{KeyBoard, KeyBoardKey, KeyBoardNoteFreq, KeyBoardNoteU8};
use oscillator::Oscillator;
use synth::Synth;

use rui::*;

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));
    let synth_clone = synth.clone();
    let synth_clone_update = synth.clone();

    std::thread::spawn(move || loop {
        synth_clone_update.lock().unwrap().update();
    });

    KeyBoard::new()
        .num_keys(25)
        .on_key_pressed(move |key: KeyBoardKey| {
            let mut synth = synth.lock().unwrap();
            let frequency: KeyBoardNoteFreq = key.try_into().unwrap();
            let audio_source = Oscillator::sine_wave(frequency).amplify(1.0);
            let source_id: KeyBoardNoteU8 = key.try_into().unwrap();
            synth.play_source(Box::new(audio_source), source_id);
        })
        .on_key_released(move |key: KeyBoardKey| {
            let mut synth = synth_clone.lock().unwrap();
            let source_id: KeyBoardNoteU8 = key.try_into().unwrap();
            synth.release_source(source_id);
        })
        .show()
        .size([400.0, 200.0])
        .run();
}
