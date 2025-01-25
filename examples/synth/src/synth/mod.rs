use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rodio::source::Source;
use rodio::{OutputStreamHandle, Sink};

/// Represents the ADSR envelope parameters
#[derive(Clone, Debug)]
pub struct EnvelopeConfig {
    attack: Duration,
    decay: Duration,
    sustain_level: f32,
    release: Duration,
}

impl EnvelopeConfig {
    pub fn builder() -> EnvelopeConfigBuilder {
        EnvelopeConfigBuilder::new()
    }
}

/// Builder for creating flexible envelope configurations
pub struct EnvelopeConfigBuilder {
    attack: Option<Duration>,
    decay: Option<Duration>,
    sustain_level: Option<f32>,
    release: Option<Duration>,
}

impl EnvelopeConfigBuilder {
    fn new() -> Self {
        Self {
            attack: None,
            decay: None,
            sustain_level: None,
            release: None,
        }
    }

    pub fn attack(mut self, duration: Duration) -> Self {
        self.attack = Some(duration);
        self
    }

    pub fn decay(mut self, duration: Duration) -> Self {
        self.decay = Some(duration);
        self
    }

    pub fn sustain(mut self, level: f32) -> Self {
        self.sustain_level = Some(level.clamp(0.0, 1.0));
        self
    }

    pub fn release(mut self, duration: Duration) -> Self {
        self.release = Some(duration);
        self
    }

    pub fn build(self) -> Result<EnvelopeConfig, &'static str> {
        Ok(EnvelopeConfig {
            attack: self.attack.unwrap_or(Duration::from_millis(50)),
            decay: self.decay.unwrap_or(Duration::from_millis(100)),
            sustain_level: self.sustain_level.unwrap_or(0.7),
            release: self.release.unwrap_or(Duration::from_millis(200)),
        })
    }
}

/// Advanced synthesizer with improved envelope and audio management
pub struct AdvancedSynth {
    audio_sinks: HashMap<u8, Sink>,
    envelope_states: HashMap<u8, EnvelopeState>,
    stream_handle: OutputStreamHandle,
    default_envelope: EnvelopeConfig,
}

struct EnvelopeState {
    config: EnvelopeConfig,
    start_time: Instant,
    is_releasing: bool,
    release_start_time: Option<Instant>,
}

impl AdvancedSynth {
    pub fn new(
        stream_handle: OutputStreamHandle,
        default_envelope: Option<EnvelopeConfig>,
    ) -> Self {
        AdvancedSynth {
            audio_sinks: HashMap::new(),
            envelope_states: HashMap::new(),
            stream_handle,
            default_envelope: default_envelope.unwrap_or_else(|| {
                EnvelopeConfig::builder()
                    .attack(Duration::from_millis(50))
                    .decay(Duration::from_millis(100))
                    .sustain(0.7)
                    .release(Duration::from_millis(200))
                    .build()
                    .expect("Default envelope configuration failed")
            }),
        }
    }

    pub fn play_source(
        &mut self,
        audio_source: Box<dyn Source<Item = f32> + Send>,
        source_id: u8,
        envelope: Option<EnvelopeConfig>,
    ) -> Result<(), &'static str> {
        let sink = Sink::try_new(&self.stream_handle).map_err(|_| "Failed to create audio sink")?;

        sink.append(audio_source);

        let envelope_state = EnvelopeState {
            config: envelope.unwrap_or_else(|| self.default_envelope.clone()),
            start_time: Instant::now(),
            is_releasing: false,
            release_start_time: None,
        };

        self.audio_sinks.insert(source_id, sink);
        self.envelope_states.insert(source_id, envelope_state);

        Ok(())
    }

    pub fn release_source(&mut self, source_id: u8) -> Result<(), &'static str> {
        self.envelope_states
            .get_mut(&source_id)
            .map(|state| {
                state.is_releasing = true;
                state.release_start_time = Some(Instant::now());
            })
            .ok_or("Source not found")
    }

    pub fn update(&mut self) {
        let now = Instant::now();
        let mut sources_to_remove = Vec::new();

        for (source_id, envelope_state) in self.envelope_states.iter_mut() {
            let sink = match self.audio_sinks.get_mut(source_id) {
                Some(sink) => sink,
                None => continue,
            };

            let volume = calculate_envelope_volume(now, envelope_state);
            sink.set_volume(volume);

            if is_source_completed(now, envelope_state) {
                sources_to_remove.push(*source_id);
            }
        }

        for source_id in sources_to_remove {
            self.envelope_states.remove(&source_id);
            self.audio_sinks.remove(&source_id);
        }
    }
}

fn calculate_envelope_volume(now: Instant, envelope_state: &EnvelopeState) -> f32 {
    let elapsed = now.duration_since(envelope_state.start_time);
    let config = &envelope_state.config;

    match (elapsed, envelope_state.is_releasing) {
        (t, false) if t < config.attack => (t.as_secs_f32() / config.attack.as_secs_f32()).min(1.0),

        (t, false) if t < config.attack + config.decay => {
            let decay_elapsed = t - config.attack;
            1.0 - (decay_elapsed.as_secs_f32() / config.decay.as_secs_f32())
                * (1.0 - config.sustain_level)
        }

        (_, true) => {
            let release_time = now.duration_since(envelope_state.release_start_time.unwrap());
            (config.sustain_level
                * (1.0 - release_time.as_secs_f32() / config.release.as_secs_f32()))
            .max(0.0)
        }

        _ => config.sustain_level,
    }
}

fn is_source_completed(now: Instant, envelope_state: &EnvelopeState) -> bool {
    envelope_state.is_releasing
        && now.duration_since(envelope_state.release_start_time.unwrap())
            >= envelope_state.config.release
}
