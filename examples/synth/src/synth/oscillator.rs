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
    oversampling: usize,
    zero_crossings: usize,
}

impl Default for OscillatorConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            anti_aliasing: true,
            oversampling: 256,
            zero_crossings: 16,
        }
    }
}

#[allow(dead_code)]
impl OscillatorConfig {
    /// Create a new configuration with custom parameters
    pub fn new() -> Self {
        Self::default()
    }

    /// Set sample rate
    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    /// Enable or disable anti-aliasing
    pub fn anti_aliasing(mut self, enabled: bool) -> Self {
        self.anti_aliasing = enabled;
        self
    }

    /// Set oversampling rate
    pub fn oversampling(mut self, rate: usize) -> Self {
        self.oversampling = rate;
        self
    }

    /// Set number of zero crossings
    pub fn zero_crossings(mut self, crossings: usize) -> Self {
        self.zero_crossings = crossings;
        self
    }
}

/// BLEP (Band-Limited Step) Table Builder
pub struct BlepTableBuilder {
    oversampling: usize,
    zero_crossings: usize,
}

impl BlepTableBuilder {
    /// Create a new builder with custom configuration
    pub fn new() -> Self {
        Self {
            oversampling: 256,
            zero_crossings: 16,
        }
    }

    /// Set the oversampling rate
    pub fn oversampling(mut self, rate: usize) -> Self {
        self.oversampling = rate;
        self
    }

    /// Set the number of zero crossings
    pub fn zero_crossings(mut self, crossings: usize) -> Self {
        self.zero_crossings = crossings;
        self
    }

    /// Generate the BLEP table with the current configuration
    pub fn generate(self) -> Vec<f32> {
        let blep_size = self.oversampling * self.zero_crossings * 2 + 1;
        (0..blep_size)
            .map(|i| {
                let x = (i as f32 / blep_size as f32 - 0.5) * self.zero_crossings as f32;
                if x == 0.0 {
                    1.0
                } else {
                    x.sin() / (std::f32::consts::PI * x)
                        * (1.0 - (x / self.zero_crossings as f32).cos())
                }
            })
            .collect()
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
    /// Create a new analog oscillator with custom configuration
    pub fn new(config: OscillatorConfig) -> Self {
        let nyquist = config.sample_rate as f32 / 2.0;
        let blep_table = BlepTableBuilder::new()
            .oversampling(config.oversampling)
            .zero_crossings(config.zero_crossings)
            .generate();

        Self {
            config,
            phase: 0.0,
            nyquist,
            phase_increment: 0.0,
            blep_table,
        }
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
        let mut output = if self.phase < pw { 1.0 } else { -1.0 };

        if self.config.anti_aliasing {
            self.apply_blep(&mut output);
        }

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        output
    }

    /// Apply band-limited step correction
    fn apply_blep(&mut self, output: &mut f32) {
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

#[allow(dead_code)]
impl Oscillator {
    /// Create oscillators for different wave types with default configuration
    pub fn sine(freq: f32) -> Self {
        Self::new(freq, WaveType::Sine, OscillatorConfig::default())
    }

    pub fn square(freq: f32) -> Self {
        Self::new(freq, WaveType::Square, OscillatorConfig::default())
    }

    pub fn sawtooth(freq: f32) -> Self {
        Self::new(freq, WaveType::Sawtooth, OscillatorConfig::default())
    }

    pub fn triangle(freq: f32) -> Self {
        Self::new(freq, WaveType::Triangle, OscillatorConfig::default())
    }

    /// Internal constructor for oscillators
    fn new(freq: f32, wave_type: WaveType, config: OscillatorConfig) -> Self {
        Oscillator {
            freq,
            num_samples: 0,
            wave_type,
            analog_osc: AnalogOsc::new(config),
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_samples = self.num_samples.wrapping_add(1);

        Some(match self.wave_type {
            WaveType::Sine => (2.0 * PI * self.freq * self.num_samples as f32
                / self.analog_osc.config.sample_rate as f32)
                .sin(),
            WaveType::Square => self.analog_osc.generate_square(self.freq, 0.5),
            WaveType::Sawtooth => self.analog_osc.generate_sawtooth(self.freq),
            WaveType::Triangle => {
                // Derive triangle wave from sine wave
                let sin_val = (2.0 * PI * self.freq * self.num_samples as f32
                    / self.analog_osc.config.sample_rate as f32)
                    .sin();
                sin_val.asin() * 2.0 / PI
            }
        })
    }
}

impl rodio::Source for Oscillator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.analog_osc.config.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
