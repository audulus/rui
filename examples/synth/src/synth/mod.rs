use std::collections::HashMap;
use std::time::{Duration, Instant};

use rodio::source::Source;
use rodio::Sink;

mod oscillator;
pub use oscillator::Oscillator;

// The envelope state struct
struct EnvelopeState {
    envelope: Envelope,
    start_time: Instant,
    is_releasing: bool,
    release_start_time: Option<Instant>,
}

// The envelope struct
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
}

impl Synth {
    pub fn new(stream_handle: rodio::OutputStreamHandle) -> Synth {
        // let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        // For some reason the above code would fail if it was in the new function

        Synth {
            audio_sinks: HashMap::new(),
            envelope_states: HashMap::new(),
            stream_handle,
        }
    }

    pub fn play_source(&mut self, audio_source: Box<dyn Source<Item = f32> + Send>, source_id: u8) {
        let sink = Sink::try_new(&self.stream_handle).expect("Failed to create sink");
        sink.append(audio_source);

        let envelope = Envelope::new(0.1, 0.2, 0.7, 1.3); // example envelope
        let envelope_state = EnvelopeState {
            envelope,
            start_time: Instant::now(),
            is_releasing: false,
            release_start_time: None,
        };

        self.audio_sinks.insert(source_id, sink);
        self.envelope_states.insert(source_id, envelope_state);
    }

    pub fn release_source(&mut self, source_id: u8) {
        if let Some(envelope_state) = self.envelope_states.get_mut(&source_id) {
            envelope_state.is_releasing = true;
            envelope_state.release_start_time = Some(Instant::now());
        }
    }

    pub fn update(&mut self) {
        let now = Instant::now();

        let mut to_remove = Vec::new();

        for (source_id, envelope_state) in self.envelope_states.iter_mut() {
            let elapsed = now.duration_since(envelope_state.start_time).as_secs_f32();

            let envelope = &envelope_state.envelope;
            let sink = self.audio_sinks.get_mut(source_id).unwrap();

            let volume = if elapsed < envelope.attack {
                // Attack
                elapsed / envelope.attack
            } else if elapsed < envelope.attack + envelope.decay {
                // Decay
                1.0 - (elapsed - envelope.attack) / envelope.decay * (1.0 - envelope.sustain)
            } else if envelope_state.is_releasing {
                // Release
                let elapsed_since_released = now
                    .duration_since(envelope_state.release_start_time.unwrap())
                    .as_secs_f32();
                envelope.sustain - elapsed_since_released / envelope.release * envelope.sustain
            } else {
                // Sustain
                envelope.sustain
            };

            sink.set_volume(volume);

            if envelope_state.is_releasing && elapsed > envelope.release {
                // This is done as a separate step to avoid a second mutable borrow of self.envelope_states
                // First borrow is when .iter_mut() is called, second is when .remove() is called
                to_remove.push(*source_id);
            }
        }

        for source_id in to_remove {
            self.envelope_states.remove(&source_id);
            self.audio_sinks.remove(&source_id);
        }
    }
}
