use std::collections::HashMap;
use std::time::Instant;

use rodio::source::Source;
use rodio::Sink;

mod oscillator;
pub use oscillator::{Oscillator, WaveType};

/// All user-controllable synth parameters.
#[derive(Clone, Debug)]
pub struct SynthParams {
    pub wave_type: WaveType,
    pub attack: f32,
    pub decay: f32,
    pub sustain: f32,
    pub release: f32,
    pub gain: f32,
    pub octave_offset: i8,
    pub filter_cutoff: f32,
    pub filter_resonance: f32,
    pub detune_cents: f32,
    pub unison_voices: u8,
}

impl Default for SynthParams {
    fn default() -> Self {
        Self {
            wave_type: WaveType::Sawtooth,
            attack: 0.01,
            decay: 0.2,
            sustain: 0.7,
            release: 0.5,
            gain: 0.7,
            octave_offset: 0,
            filter_cutoff: 20000.0,
            filter_resonance: 0.0,
            detune_cents: 0.0,
            unison_voices: 1,
        }
    }
}

struct EnvelopeState {
    envelope: Envelope,
    start_time: Instant,
    is_releasing: bool,
    release_start_time: Option<Instant>,
    release_start_volume: Option<f32>,
}

struct Envelope {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
}

impl Envelope {
    fn new(attack: f32, decay: f32, sustain: f32, release: f32) -> Envelope {
        Envelope {
            attack,
            decay,
            sustain,
            release,
        }
    }
}

pub struct Synth {
    audio_sinks: HashMap<u8, Sink>,
    envelope_states: HashMap<u8, EnvelopeState>,
    stream_handle: rodio::OutputStreamHandle,
    pub params: SynthParams,
}

impl Synth {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Synth {
        Synth {
            audio_sinks: HashMap::new(),
            envelope_states: HashMap::new(),
            stream_handle,
            params: SynthParams::default(),
        }
    }

    pub fn play_source(&mut self, audio_source: Box<dyn Source<Item = f32> + Send>, source_id: u8) {
        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(audio_source);

        let envelope = Envelope::new(
            self.params.attack,
            self.params.decay,
            self.params.sustain,
            self.params.release,
        );
        let envelope_state = EnvelopeState {
            envelope,
            start_time: Instant::now(),
            is_releasing: false,
            release_start_time: None,
            release_start_volume: None,
        };

        self.audio_sinks.insert(source_id, sink);
        self.envelope_states.insert(source_id, envelope_state);
    }

    pub fn release_source(&mut self, source_id: u8) {
        if let Some(envelope_state) = self.envelope_states.get_mut(&source_id) {
            let now = Instant::now();
            let elapsed = now.duration_since(envelope_state.start_time).as_secs_f32();
            let envelope = &envelope_state.envelope;

            let current_volume = if elapsed < envelope.attack {
                elapsed / envelope.attack
            } else if elapsed < envelope.attack + envelope.decay {
                1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
            } else {
                envelope.sustain
            };

            envelope_state.is_releasing = true;
            envelope_state.release_start_time = Some(now);
            envelope_state.release_start_volume = Some(current_volume);
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let mut to_remove = Vec::new();

        for (source_id, envelope_state) in self.envelope_states.iter_mut() {
            let sink = self.audio_sinks.get_mut(source_id).unwrap();
            let envelope = &envelope_state.envelope;

            let volume = if envelope_state.is_releasing {
                let elapsed_release = now
                    .duration_since(envelope_state.release_start_time.unwrap())
                    .as_secs_f32();

                let start_volume = envelope_state.release_start_volume.unwrap();
                let t = (elapsed_release / envelope.release).min(1.0);
                start_volume * (1.0 - t)
            } else {
                let elapsed = now.duration_since(envelope_state.start_time).as_secs_f32();

                if elapsed < envelope.attack {
                    elapsed / envelope.attack
                } else if elapsed < envelope.attack + envelope.decay {
                    1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
                } else {
                    envelope.sustain
                }
            };

            sink.set_volume(volume);

            if envelope_state.is_releasing {
                let elapsed_release = now
                    .duration_since(envelope_state.release_start_time.unwrap())
                    .as_secs_f32();

                if elapsed_release >= envelope.release {
                    to_remove.push(*source_id);
                }
            }
        }

        for source_id in to_remove {
            self.envelope_states.remove(&source_id);
            self.audio_sinks.remove(&source_id);
        }
    }
}
