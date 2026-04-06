use rodio::source::Source;
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

mod midi_keyboard;
mod synth;
mod ui;

use midi_keyboard::MidiKeyboard;
use synth::{Oscillator, Synth};
use ui::{synth_controls, SynthUI};

use rui::*;

fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let synth = Arc::new(Mutex::new(Synth::new(stream_handle)));

    // Background thread for envelope updates.
    let synth_update = synth.clone();
    std::thread::spawn(move || loop {
        synth_update.lock().unwrap().update();
        std::thread::sleep(std::time::Duration::from_millis(1));
    });

    state(SynthUI::default, move |ui_handle, cx| {
        // Sync initial state.
        if let Ok(mut s) = synth.lock() {
            s.params = cx[ui_handle].to_params();
        }

        let synth_sync = synth.clone();
        let synth_begin = synth.clone();
        let synth_end = synth.clone();

        vstack((
            synth_controls(ui_handle, move |cx| {
                if let Ok(mut s) = synth_sync.lock() {
                    s.params = cx[ui_handle].to_params();
                }
            }),
            MidiKeyboard::new()
                .num_keys(25)
                .start_octave(4)
                .max_simultaneous_keys(5)
                .on_note_begin(move |event| {
                    let mut synth = synth_begin.lock().unwrap();
                    let params = synth.params.clone();
                    let freq = event.note.frequency()
                        * 2.0_f32.powi(params.octave_offset as i32);
                    let source =
                        Oscillator::with_params(freq, &params).amplify(params.gain);
                    synth.play_source(Box::new(source), event.note.id());
                })
                .on_note_end(move |event| {
                    let mut synth = synth_end.lock().unwrap();
                    synth.release_source(event.note.id());
                })
                .show(),
        ))
    })
    .run();
}
