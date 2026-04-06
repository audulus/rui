use rodio::source::Source;
use rodio::OutputStream;
use std::sync::{Arc, Mutex};

mod midi_keyboard;
mod synth;

use midi_keyboard::MidiKeyboard;
use synth::{Oscillator, Synth, SynthParams, WaveType};

use rui::*;

/// UI state mirroring synth params with 0-1 slider values.
#[derive(Clone)]
struct SynthUI {
    wave_idx: usize,
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    gain: f32,
    octave_offset: i8,
    filter_cutoff: f32,
    filter_resonance: f32,
    detune: f32,
    unison_voices: u8,
}

impl Default for SynthUI {
    fn default() -> Self {
        Self {
            wave_idx: 2, // Sawtooth
            attack: 0.0,
            decay: 0.35,
            sustain: 0.7,
            release: 0.35,
            gain: 0.7,
            octave_offset: 0,
            filter_cutoff: 1.0,
            filter_resonance: 0.0,
            detune: 0.0,
            unison_voices: 1,
        }
    }
}

impl SynthUI {
    /// Convert UI slider values to real synth parameters.
    fn to_params(&self) -> SynthParams {
        SynthParams {
            wave_type: WaveType::ALL[self.wave_idx],
            // Exponential mapping: 0.005s to 2.0s
            attack: 0.005 * 400.0_f32.powf(self.attack),
            decay: 0.005 * 400.0_f32.powf(self.decay),
            sustain: self.sustain,
            // Release: 0.01s to 5.0s
            release: 0.01 * 500.0_f32.powf(self.release),
            gain: self.gain,
            octave_offset: self.octave_offset,
            // Cutoff: 20Hz to 20kHz exponential
            filter_cutoff: 20.0 * 1000.0_f32.powf(self.filter_cutoff),
            filter_resonance: self.filter_resonance,
            detune_cents: self.detune * 50.0,
            unison_voices: self.unison_voices,
        }
    }

    /// Sync UI state to the synth (behind mutex).
    fn sync(&self, synth: &Arc<Mutex<Synth>>) {
        if let Ok(mut s) = synth.lock() {
            s.params = self.to_params();
        }
    }
}

fn wave_button(
    _name: &'static str,
    idx: usize,
    ui_handle: StateHandle<SynthUI>,
    synth: Arc<Mutex<Synth>>,
) -> impl View {
    with_cx(move |cx| {
        let is_active = cx[ui_handle].wave_idx == idx;
        let color = if is_active {
            AZURE_HIGHLIGHT
        } else {
            CONTROL_BACKGROUND
        };
        let synth = synth.clone();
        rectangle()
            .corner_radius(4.0)
            .color(color)
            .size([40.0, 25.0])
            .tap(move |cx| {
                cx[ui_handle].wave_idx = idx;
                cx[ui_handle].sync(&synth);
            })
            .padding(2.0)
    })
    .padding(1.0)
}

fn wave_label(name: &'static str) -> impl View {
    text(name).font_size(10).padding(Auto)
}

fn labeled_param(
    label: &'static str,
    field: impl Fn(&SynthUI) -> f32 + Copy + 'static,
    set_field: impl Fn(&mut SynthUI, f32) + Copy + 'static,
    ui_handle: StateHandle<SynthUI>,
    synth: Arc<Mutex<Synth>>,
) -> impl View {
    with_cx(move |cx| {
        let val = field(&cx[ui_handle]);
        let synth = synth.clone();
        hstack((
            text(label).font_size(10).size([55.0, 18.0]),
            map(
                val,
                move |v, cx| {
                    set_field(&mut cx[ui_handle], v);
                    cx[ui_handle].sync(&synth);
                },
                |s, _| hslider(s),
            ),
        ))
        .padding(2.0)
    })
}

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
        // Clone Arcs inside the Fn closure body so they can be moved into child closures.
        let synth_for_waves = synth.clone();
        let synth_for_adsr = synth.clone();
        let synth_for_filter = synth.clone();
        let synth_for_unison = synth.clone();
        let synth_for_unison2 = synth.clone();
        let synth_for_gain = synth.clone();
        let synth_for_oct = synth.clone();
        let synth_for_oct2 = synth.clone();
        let synth_begin = synth.clone();
        let synth_end = synth.clone();

        // Sync initial state.
        cx[ui_handle].sync(&synth);

        vstack((
            // --- Top row: Waveform, Octave, Gain ---
            hstack((
                // Waveform selector
                hstack((
                    text("Wave").font_size(10).padding(Auto),
                    zstack((
                        hstack((
                            wave_button("Sin", 0, ui_handle, synth_for_waves.clone()),
                            wave_button("Sq", 1, ui_handle, synth_for_waves.clone()),
                            wave_button("Saw", 2, ui_handle, synth_for_waves.clone()),
                            wave_button("Tri", 3, ui_handle, synth_for_waves.clone()),
                        )),
                        // Labels on top of buttons
                        hstack((
                            wave_label("Sin"),
                            wave_label("Sq"),
                            wave_label("Saw"),
                            wave_label("Tri"),
                        )),
                    )),
                ))
                .padding(5.0),
                // Octave selector
                hstack((
                    text("Oct").font_size(10).padding(Auto),
                    rectangle()
                        .corner_radius(4.0)
                        .color(CONTROL_BACKGROUND)
                        .size([25.0, 25.0])
                        .tap(move |cx| {
                            cx[ui_handle].octave_offset =
                                (cx[ui_handle].octave_offset - 1).max(-2);
                            cx[ui_handle].sync(&synth_for_oct);
                        })
                        .padding(2.0),
                    with_ref(ui_handle, |ui| {
                        text(&format!("{}", 4 + ui.octave_offset))
                            .font_size(14)
                            .padding(Auto)
                    }),
                    rectangle()
                        .corner_radius(4.0)
                        .color(CONTROL_BACKGROUND)
                        .size([25.0, 25.0])
                        .tap(move |cx| {
                            cx[ui_handle].octave_offset =
                                (cx[ui_handle].octave_offset + 1).min(2);
                            cx[ui_handle].sync(&synth_for_oct2);
                        })
                        .padding(2.0),
                ))
                .padding(5.0),
                // Gain slider
                hstack((
                    text("Gain").font_size(10).size([35.0, 18.0]),
                    map(
                        cx[ui_handle].gain,
                        move |v, cx| {
                            cx[ui_handle].gain = v;
                            cx[ui_handle].sync(&synth_for_gain);
                        },
                        |s, _| hslider(s),
                    ),
                ))
                .padding(5.0),
            ))
            .size([700.0, 40.0]),
            // --- Middle row: ADSR, Filter, Unison ---
            hstack((
                // ADSR
                vstack((
                    text("Envelope").font_size(10).padding(Auto),
                    labeled_param(
                        "Attack",
                        |ui| ui.attack,
                        |ui, v| ui.attack = v,
                        ui_handle,
                        synth_for_adsr.clone(),
                    ),
                    labeled_param(
                        "Decay",
                        |ui| ui.decay,
                        |ui, v| ui.decay = v,
                        ui_handle,
                        synth_for_adsr.clone(),
                    ),
                    labeled_param(
                        "Sustain",
                        |ui| ui.sustain,
                        |ui, v| ui.sustain = v,
                        ui_handle,
                        synth_for_adsr.clone(),
                    ),
                    labeled_param(
                        "Release",
                        |ui| ui.release,
                        |ui, v| ui.release = v,
                        ui_handle,
                        synth_for_adsr.clone(),
                    ),
                ))
                .padding(5.0),
                // Filter
                vstack((
                    text("Filter").font_size(10).padding(Auto),
                    labeled_param(
                        "Cutoff",
                        |ui| ui.filter_cutoff,
                        |ui, v| ui.filter_cutoff = v,
                        ui_handle,
                        synth_for_filter.clone(),
                    ),
                    labeled_param(
                        "Reso",
                        |ui| ui.filter_resonance,
                        |ui, v| ui.filter_resonance = v,
                        ui_handle,
                        synth_for_filter.clone(),
                    ),
                ))
                .padding(5.0),
                // Unison
                vstack((
                    text("Unison").font_size(10).padding(Auto),
                    labeled_param(
                        "Detune",
                        |ui| ui.detune,
                        |ui, v| ui.detune = v,
                        ui_handle,
                        synth_for_unison.clone(),
                    ),
                    hstack((
                        text("Voices").font_size(10).size([55.0, 18.0]),
                        rectangle()
                            .corner_radius(4.0)
                            .color(CONTROL_BACKGROUND)
                            .size([25.0, 20.0])
                            .tap(move |cx| {
                                let v = cx[ui_handle].unison_voices;
                                cx[ui_handle].unison_voices = if v <= 1 { 1 } else { v - 2 };
                                cx[ui_handle].sync(&synth_for_unison);
                            })
                            .padding(2.0),
                        with_ref(ui_handle, |ui| {
                            text(&format!("{}", ui.unison_voices))
                                .font_size(12)
                                .padding(Auto)
                        }),
                        rectangle()
                            .corner_radius(4.0)
                            .color(CONTROL_BACKGROUND)
                            .size([25.0, 20.0])
                            .tap(move |cx| {
                                let v = cx[ui_handle].unison_voices;
                                cx[ui_handle].unison_voices = (v + 2).min(7);
                                cx[ui_handle].sync(&synth_for_unison2);
                            })
                            .padding(2.0),
                    ))
                    .padding(2.0),
                ))
                .padding(5.0),
            ))
            .size([700.0, 130.0]),
            // --- Keyboard ---
            MidiKeyboard::new()
                .num_keys(25)
                .start_octave(4)
                .max_simultaneous_keys(5)
                .on_note_begin(move |event| {
                    let mut synth = synth_begin.lock().unwrap();
                    let params = synth.params.clone();
                    let freq = event.note.frequency()
                        * 2.0_f32.powi(params.octave_offset as i32);
                    let source = Oscillator::with_params(freq, &params)
                        .amplify(params.gain);
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
