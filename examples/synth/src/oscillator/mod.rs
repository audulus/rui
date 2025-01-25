use rodio::source::Source;
use std::f32::consts::PI;

/// Represents different wave types for audio synthesis
#[derive(Clone, Debug)]
pub enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

/// Configuration parameters for oscillator generation
#[derive(Clone, Debug)]
pub struct OscillatorConfig {
    sample_rate: u32,
    anti_aliasing: bool,
}

impl Default for OscillatorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            anti_aliasing: true,
        }
    }
}

/// Advanced analog oscillator with high-quality wave generation
#[derive(Clone, Debug)]
pub struct AnalogOsc {
    config: OscillatorConfig,
    phase: f32,
    nyquist: f32,
    phase_increment: f32,
    blep_table: Vec<f32>,
}

impl AnalogOsc {
    /// Create a new analog oscillator with default configuration
    pub fn new() -> Self {
        let config = OscillatorConfig::default();
        let nyquist = config.sample_rate as f32 / 2.0;
        Self {
            config,
            phase: 0.0,
            nyquist,
            phase_increment: 0.0,
            blep_table: Self::generate_blep_table(),
        }
    }

    /// Generate a high-quality BLEP (Band-Limited Step) table
    fn generate_blep_table() -> Vec<f32> {
        const BLEP_SIZE: usize = 1024;
        const ZERO_CROSSINGS: usize = 16;

        (0..BLEP_SIZE)
            .map(|i| {
                let x = (i as f32 / BLEP_SIZE as f32 - 0.5) * ZERO_CROSSINGS as f32;
                if x == 0.0 {
                    1.0
                } else {
                    x.sin() / (PI * x) * (1.0 - (x / ZERO_CROSSINGS as f32).cos())
                }
            })
            .collect()
    }

    /// Generate a sawtooth wave sample with optional anti-aliasing
    pub fn generate_sawtooth(&mut self, frequency: f32) -> f32 {
        let frequency = frequency.min(self.nyquist);
        self.phase_increment = frequency / self.config.sample_rate as f32;

        let mut output = 2.0 * self.phase - 1.0;

        if self.config.anti_aliasing {
            self.apply_blep(&mut output);
        }

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    /// Generate a square wave sample with optional anti-aliasing
    pub fn generate_square(&mut self, frequency: f32, pulse_width: f32) -> f32 {
        let frequency = frequency.min(self.nyquist);
        self.phase_increment = frequency / self.config.sample_rate as f32;

        let pw = pulse_width.clamp(0.0, 1.0);
        let output = if self.phase < pw { 1.0 } else { -1.0 };

        if self.config.anti_aliasing {
            self.apply_blep(&mut output.clone());
        }

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    /// Apply band-limited step correction
    fn apply_blep(&mut self, output: &mut f32) {
        // Basic BLEP application - this is a simplified implementation
        // A more advanced version would interpolate and add correction
        if self.phase < self.phase_increment {
            let index = (self.phase / self.phase_increment * self.blep_table.len() as f32) as usize;
            *output += self.blep_table.get(index).cloned().unwrap_or(0.0);
        }
    }
}

/// Implements a flexible audio oscillator for different wave types
pub struct Oscillator {
    freq: f32,
    num_samples: usize,
    wave_type: WaveType,
    analog_osc: AnalogOsc,
}

impl Oscillator {
    /// Create oscillators for different wave types
    pub fn sine(freq: f32) -> Self {
        Self::new(freq, WaveType::Sine)
    }

    pub fn square(freq: f32) -> Self {
        Self::new(freq, WaveType::Square)
    }

    pub fn sawtooth(freq: f32) -> Self {
        Self::new(freq, WaveType::Sawtooth)
    }

    pub fn triangle(freq: f32) -> Self {
        Self::new(freq, WaveType::Triangle)
    }

    /// Internal constructor for oscillators
    fn new(freq: f32, wave_type: WaveType) -> Self {
        Oscillator {
            freq,
            num_samples: 0,
            wave_type,
            analog_osc: AnalogOsc::new(),
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_samples = self.num_samples.wrapping_add(1);

        Some(match self.wave_type {
            WaveType::Sine => (2.0 * PI * self.freq * self.num_samples as f32 / 48000.0).sin(),
            WaveType::Square => self.analog_osc.generate_square(self.freq, 0.5),
            WaveType::Sawtooth => self.analog_osc.generate_sawtooth(self.freq),
            WaveType::Triangle => {
                // Derive triangle wave from sine wave
                let sin_val = (2.0 * PI * self.freq * self.num_samples as f32 / 48000.0).sin();
                sin_val.asin() * 2.0 / PI
            }
        })
    }
}

impl Source for Oscillator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        48000
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
