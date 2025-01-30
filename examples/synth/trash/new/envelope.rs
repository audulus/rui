use std::cell::Cell;
use std::time::{Duration, Instant};

/// Configuration for a standard ADSR (Attack, Decay, Sustain, Release) envelope
#[derive(Clone, Debug)]
pub struct ADSREnvelopeConfig {
    attack: Duration,
    decay: Duration,
    sustain_level: f32,
    release: Duration,
}

impl ADSREnvelopeConfig {
    pub fn builder() -> ADSREnvelopeConfigBuilder {
        ADSREnvelopeConfigBuilder::new()
    }
}

/// Builder for creating flexible ADSR envelope configurations
pub struct ADSREnvelopeConfigBuilder {
    attack: Option<Duration>,
    decay: Option<Duration>,
    sustain_level: Option<f32>,
    release: Option<Duration>,
}

impl ADSREnvelopeConfigBuilder {
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

    pub fn build(self) -> Result<ADSREnvelopeConfig, &'static str> {
        Ok(ADSREnvelopeConfig {
            attack: self.attack.unwrap_or(Duration::from_millis(50)),
            decay: self.decay.unwrap_or(Duration::from_millis(100)),
            sustain_level: self.sustain_level.unwrap_or(0.7),
            release: self.release.unwrap_or(Duration::from_millis(200)),
        })
    }
}

/// Concrete implementation of an ADSR envelope
pub struct ADSREnvelope {
    config: ADSREnvelopeConfig,
    start_time: Cell<Option<Instant>>,
    release_start_time: Cell<Option<Instant>>,
    release_start_amplitude: Cell<f32>,
}

impl ADSREnvelope {
    pub fn new(config: ADSREnvelopeConfig) -> Self {
        Self {
            config,
            start_time: Cell::new(None),
            release_start_time: Cell::new(None),
            release_start_amplitude: Cell::new(0.0),
        }
    }

    pub fn start(&self) {
        self.start_time.set(None);
        self.release_start_time.set(None);
        self.release_start_amplitude.set(0.0);
    }

    pub fn release(&self) {
        if self.start_time.get().is_some() && self.release_start_time.get().is_none() {
            let now = Instant::now();
            self.release_start_time.set(Some(now));
            self.release_start_amplitude
                .set(self.get_amplitude_internal(now, false));
        }
    }

    pub fn get_amplitude(&self, now: Instant) -> f32 {
        self.get_amplitude_internal(now, true)
    }

    fn get_amplitude_internal(&self, now: Instant, clamp: bool) -> f32 {
        // Set start_time if not set
        let start_time = match self.start_time.get() {
            Some(st) => st,
            None => {
                self.start_time.set(Some(now));
                return 0.0;
            }
        };

        let elapsed = now.duration_since(start_time);

        // Determine if in release phase
        let is_releasing = self.release_start_time.get().is_some();

        match (elapsed, is_releasing) {
            // Attack phase: Linear ramp from 0 to 1
            (t, false) if t < self.config.attack => {
                let amp = t.as_secs_f32() / self.config.attack.as_secs_f32();
                if clamp {
                    amp.clamp(0.0, 1.0)
                } else {
                    amp
                }
            }

            // Decay phase: Linear ramp from 1 to sustain level
            (t, false) if t < self.config.attack + self.config.decay => {
                let decay_progress =
                    (t - self.config.attack).as_secs_f32() / self.config.decay.as_secs_f32();
                let amp = 1.0 - decay_progress * (1.0 - self.config.sustain_level);
                if clamp {
                    amp.clamp(self.config.sustain_level, 1.0)
                } else {
                    amp
                }
            }

            // Release phase: Linear ramp from current amplitude to 0
            (_, true) => {
                let Some(release_start) = self.release_start_time.get() else {
                    return self.config.sustain_level;
                };

                let release_time = now.duration_since(release_start);
                let release_progress =
                    release_time.as_secs_f32() / self.config.release.as_secs_f32();
                let amp = self.release_start_amplitude.get() * (1.0 - release_progress);
                if clamp {
                    amp.clamp(0.0, self.release_start_amplitude.get())
                } else {
                    amp
                }
            }

            // Sustain phase
            _ => self.config.sustain_level,
        }
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        match (self.start_time.get(), self.release_start_time.get()) {
            (Some(_), Some(release_start)) => {
                now.duration_since(release_start) >= self.config.release
            }
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adsr_envelope() {
        let config = ADSREnvelopeConfig::builder()
            .attack(Duration::from_millis(100))
            .decay(Duration::from_millis(50))
            .sustain(0.5)
            .release(Duration::from_millis(100))
            .build()
            .unwrap();

        let envelope = ADSREnvelope::new(config);

        // Start the envelope
        envelope.start();

        // Check amplitude immediately after start (should be near 0.0)
        let start = Instant::now();
        let initial_amplitude = envelope.get_amplitude(start);
        assert!(
            initial_amplitude < 1e-5,
            "Initial amplitude should be near 0.0, got {}",
            initial_amplitude
        );

        // Simulate partial attack phase
        let mid_attack = start + Duration::from_millis(50);
        let amplitude = envelope.get_amplitude(mid_attack);
        assert!(
            amplitude > 0.0 && amplitude < 1.0,
            "Amplitude during attack should be between 0 and 1"
        );

        // Simulate full attack phase completion
        let end_attack = start + Duration::from_millis(100);
        let amplitude = envelope.get_amplitude(end_attack);
        assert!(
            (amplitude - 1.0).abs() < 1e-5,
            "Amplitude at end of attack should be approximately 1.0, got {}",
            amplitude
        );

        // Test release during decay phase
        let mid_decay = start + Duration::from_millis(125);
        envelope.release();
        let release_start = envelope.release_start_time.get().unwrap();
        let release_progress = mid_decay.duration_since(release_start);
        let amplitude = envelope.get_amplitude(mid_decay);
        assert!(
            amplitude < 1.0 && amplitude > 0.0,
            "Amplitude should start from current level and release"
        );
    }
}
