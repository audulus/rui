use crate::midi_keyboard::MidiKeyboard;
use crate::synth::{SynthParams, WaveType};
use rui::*;

/// UI state mirroring synth params with 0-1 slider values.
#[derive(Clone)]
pub struct SynthUI {
    pub wave_idx: usize,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub gain: f32,
    pub octave_offset: i8,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub detune: f32,
    pub unison_voices: u8,
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
    pub fn to_params(&self) -> SynthParams {
        SynthParams {
            wave_type: WaveType::ALL[self.wave_idx],
            attack: 0.005 * 400.0_f32.powf(self.attack),
            decay: 0.005 * 400.0_f32.powf(self.decay),
            sustain: self.sustain,
            release: 0.01 * 500.0_f32.powf(self.release),
            gain: self.gain,
            octave_offset: self.octave_offset,
            filter_cutoff: 20.0 * 1000.0_f32.powf(self.filter_cutoff),
            filter_resonance: self.filter_resonance,
            detune_cents: self.detune * 50.0,
            unison_voices: self.unison_voices,
        }
    }
}

// --- Reusable UI building blocks ---

fn wave_button(
    idx: usize,
    ui_handle: StateHandle<SynthUI>,
    on_change: impl Fn(&mut Context) + Clone + 'static,
) -> impl View {
    with_cx(move |cx| {
        let is_active = cx[ui_handle].wave_idx == idx;
        let color = if is_active {
            AZURE_HIGHLIGHT
        } else {
            CONTROL_BACKGROUND
        };
        let on_change = on_change.clone();
        rectangle()
            .corner_radius(4.0)
            .color(color)
            .size([40.0, 25.0])
            .tap(move |cx| {
                cx[ui_handle].wave_idx = idx;
                on_change(cx);
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
    on_change: impl Fn(&mut Context) + Clone + 'static,
) -> impl View {
    with_cx(move |cx| {
        let val = field(&cx[ui_handle]);
        let on_change = on_change.clone();
        hstack((
            text(label).font_size(10).size([55.0, 18.0]),
            map(
                val,
                move |v, cx| {
                    set_field(&mut cx[ui_handle], v);
                    on_change(cx);
                },
                |s, _| hslider(s),
            ),
        ))
        .padding(2.0)
    })
}

/// Build the synth control panel (everything above the keyboard).
/// `on_change` is called whenever a parameter changes.
pub fn synth_controls(
    ui_handle: StateHandle<SynthUI>,
    on_change: impl Fn(&mut Context) + Clone + 'static,
) -> impl View {
    let oc = on_change.clone();
    let oc2 = on_change.clone();
    let oc3 = on_change.clone();
    let oc4 = on_change.clone();
    let oc5 = on_change.clone();
    let oc6 = on_change.clone();
    let oc7 = on_change.clone();
    let oc8 = on_change.clone();
    let oc9 = on_change.clone();
    let oc10 = on_change.clone();
    let oc11 = on_change.clone();
    let oc12 = on_change.clone();

    vstack((
        // --- Top row: Waveform, Octave, Gain ---
        hstack((
            // Waveform selector
            hstack((
                text("Wave").font_size(10).padding(Auto),
                zstack((
                    hstack((
                        wave_button(0, ui_handle, oc),
                        wave_button(1, ui_handle, oc2),
                        wave_button(2, ui_handle, oc3),
                        wave_button(3, ui_handle, oc4),
                    )),
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
                        oc5(cx);
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
                        oc6(cx);
                    })
                    .padding(2.0),
            ))
            .padding(5.0),
            // Gain slider
            with_cx(move |cx| {
                let val = cx[ui_handle].gain;
                let oc = oc7.clone();
                hstack((
                    text("Gain").font_size(10).size([35.0, 18.0]),
                    map(
                        val,
                        move |v, cx| {
                            cx[ui_handle].gain = v;
                            oc(cx);
                        },
                        |s, _| hslider(s),
                    ),
                ))
                .padding(5.0)
            }),
        ))
        .size([700.0, 40.0]),
        // --- Middle row: ADSR, Filter, Unison ---
        hstack((
            // ADSR
            vstack((
                text("Envelope").font_size(10).padding(Auto),
                labeled_param("Attack", |ui| ui.attack, |ui, v| ui.attack = v, ui_handle, oc8.clone()),
                labeled_param("Decay", |ui| ui.decay, |ui, v| ui.decay = v, ui_handle, oc8.clone()),
                labeled_param("Sustain", |ui| ui.sustain, |ui, v| ui.sustain = v, ui_handle, oc8.clone()),
                labeled_param("Release", |ui| ui.release, |ui, v| ui.release = v, ui_handle, oc8),
            ))
            .padding(5.0),
            // Filter
            vstack((
                text("Filter").font_size(10).padding(Auto),
                labeled_param("Cutoff", |ui| ui.filter_cutoff, |ui, v| ui.filter_cutoff = v, ui_handle, oc9.clone()),
                labeled_param("Reso", |ui| ui.filter_resonance, |ui, v| ui.filter_resonance = v, ui_handle, oc9),
            ))
            .padding(5.0),
            // Unison
            vstack((
                text("Unison").font_size(10).padding(Auto),
                labeled_param("Detune", |ui| ui.detune, |ui, v| ui.detune = v, ui_handle, oc10),
                hstack((
                    text("Voices").font_size(10).size([55.0, 18.0]),
                    rectangle()
                        .corner_radius(4.0)
                        .color(CONTROL_BACKGROUND)
                        .size([25.0, 20.0])
                        .tap(move |cx| {
                            let v = cx[ui_handle].unison_voices;
                            cx[ui_handle].unison_voices = if v <= 1 { 1 } else { v - 2 };
                            oc11(cx);
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
                            oc12(cx);
                        })
                        .padding(2.0),
                ))
                .padding(2.0),
            ))
            .padding(5.0),
        ))
        .size([700.0, 130.0]),
    ))
}

/// Build the complete synth view (controls + keyboard) with no audio dependencies.
pub fn synth_view_no_audio(
    ui_handle: StateHandle<SynthUI>,
    on_change: impl Fn(&mut Context) + Clone + 'static,
) -> impl View {
    vstack((
        synth_controls(ui_handle, on_change),
        MidiKeyboard::new()
            .num_keys(25)
            .start_octave(4)
            .max_simultaneous_keys(5)
            .show(),
    ))
}
