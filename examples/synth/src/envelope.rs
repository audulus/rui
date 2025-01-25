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
    start_time: Option<Instant>,
    release_start_time: Option<Instant>,
}

impl ADSREnvelope {
    pub fn new(config: ADSREnvelopeConfig) -> Self {
        Self {
            config,
            start_time: None,
            release_start_time: None,
        }
    }

    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.release_start_time = None;
    }

    pub fn release(&mut self) {
        if let Some(start_time) = self.start_time {
            if self.release_start_time.is_none() {
                self.release_start_time = Some(Instant::now());
            }
        }
    }

    pub fn get_amplitude(&self, now: Instant) -> f32 {
        // Require start_time to be set
        let Some(start_time) = self.start_time else {
            return 0.0;
        };

        let elapsed = now.duration_since(start_time);

        // Determine if in release phase
        let is_releasing = self.release_start_time.is_some();

        match (elapsed, is_releasing) {
            // Attack phase: Linear ramp from 0 to 1
            (t, false) if t < self.config.attack => {
                (t.as_secs_f32() / self.config.attack.as_secs_f32()).clamp(0.0, 1.0)
            }

            // Decay phase: Linear ramp from 1 to sustain level
            (t, false) if t < self.config.attack + self.config.decay => {
                let decay_progress =
                    (t - self.config.attack).as_secs_f32() / self.config.decay.as_secs_f32();
                (1.0 - decay_progress * (1.0 - self.config.sustain_level))
                    .clamp(self.config.sustain_level, 1.0)
            }

            // Release phase: Linear ramp from sustain level to 0
            (_, true) => {
                let Some(release_start) = self.release_start_time else {
                    return self.config.sustain_level;
                };

                let release_time = now.duration_since(release_start);
                (self.config.sustain_level
                    * (1.0 - release_time.as_secs_f32() / self.config.release.as_secs_f32()))
                .clamp(0.0, self.config.sustain_level)
            }

            // Sustain phase
            _ => self.config.sustain_level,
        }
    }

    pub fn is_finished(&self, now: Instant) -> bool {
        // Envelope is finished if in release phase and release duration has passed
        match (self.start_time, self.release_start_time) {
            (Some(start), Some(release_start)) => {
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

        let mut envelope = ADSREnvelope::new(config);

        // Start the envelope
        envelope.start();

        // Immediately after start - amplitude should be 0
        let start = Instant::now();
        assert_eq!(envelope.get_amplitude(start), 0.0);

        // Simulate partial attack phase
        let mid_attack = start + Duration::from_millis(50);
        let amplitude = envelope.get_amplitude(mid_attack);
        assert!(
            amplitude > 0.0 && amplitude < 1.0,
            "Amplitude during attack should be between 0 and 1"
        );

        // Simulate full attack phase completion
        let end_attack = start + Duration::from_millis(100);
        assert_eq!(envelope.get_amplitude(end_attack), 1.0);
    }
}
