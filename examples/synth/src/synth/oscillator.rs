use std::f32::consts::PI;

use super::SynthParams;

/// Represents different wave types for audio synthesis
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

impl WaveType {
    pub const ALL: [WaveType; 4] = [
        WaveType::Sine,
        WaveType::Square,
        WaveType::Sawtooth,
        WaveType::Triangle,
    ];

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            WaveType::Sine => "Sin",
            WaveType::Square => "Sq",
            WaveType::Sawtooth => "Saw",
            WaveType::Triangle => "Tri",
        }
    }
}

/// Configuration parameters for oscillator generation
#[derive(Clone, Debug)]
pub struct OscillatorConfig {
    pub sample_rate: u32,
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

/// BLEP (Band-Limited Step) Table Builder
pub struct BlepTableBuilder {
    oversampling: usize,
    zero_crossings: usize,
}

impl BlepTableBuilder {
    pub fn new() -> Self {
        Self {
            oversampling: 256,
            zero_crossings: 16,
        }
    }

    pub fn oversampling(mut self, rate: usize) -> Self {
        self.oversampling = rate;
        self
    }

    pub fn zero_crossings(mut self, crossings: usize) -> Self {
        self.zero_crossings = crossings;
        self
    }

    pub fn generate(self) -> Vec<f32> {
        let blep_size = self.oversampling * self.zero_crossings * 2 + 1;
        (0..blep_size)
            .map(|i| {
                let x = (i as f32 / blep_size as f32 - 0.5) * self.zero_crossings as f32;
                if x == 0.0 {
                    1.0
                } else {
                    x.sin() / (PI * x) * (1.0 - (x / self.zero_crossings as f32).cos())
                }
            })
            .collect()
    }
}

// --- Moog Ladder Filter (Huovilainen improved model) ---

fn fast_tanh(x: f32) -> f32 {
    let x2 = x * x;
    x * (27.0 + x2) / (27.0 + 9.0 * x2)
}

/// 4-pole (24dB/oct) Moog-style ladder filter.
#[derive(Clone, Debug)]
pub struct MoogLadderFilter {
    stage: [f32; 4],
    stage_tanh: [f32; 3],
    tune: f32,
    resonance: f32,
    sample_rate: f32,
}

impl MoogLadderFilter {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            stage: [0.0; 4],
            stage_tanh: [0.0; 3],
            tune: 1.0,
            resonance: 0.0,
            sample_rate,
        }
    }

    /// Set filter parameters.
    /// cutoff: frequency in Hz (20-20000)
    /// resonance: 0.0-1.0 (mapped internally to 0-3.99)
    pub fn set_params(&mut self, cutoff: f32, resonance: f32) {
        let cutoff = cutoff.clamp(20.0, self.sample_rate * 0.49);
        let f = 2.0 * cutoff / self.sample_rate;
        self.tune = 1.0 - (-2.0 * PI * f).exp();
        self.resonance = (resonance * 4.0).min(3.99);
    }

    /// Process a single sample through the filter.
    pub fn process(&mut self, input: f32) -> f32 {
        // 2x oversampling to reduce nonlinear aliasing
        for _ in 0..2 {
            let inp = input - self.resonance * self.stage[3];
            self.stage[0] += self.tune * (fast_tanh(inp) - self.stage_tanh[0]);
            self.stage_tanh[0] = fast_tanh(self.stage[0]);

            self.stage[1] += self.tune * (self.stage_tanh[0] - self.stage_tanh[1]);
            self.stage_tanh[1] = fast_tanh(self.stage[1]);

            self.stage[2] += self.tune * (self.stage_tanh[1] - self.stage_tanh[2]);
            self.stage_tanh[2] = fast_tanh(self.stage[2]);

            self.stage[3] += self.tune * (self.stage_tanh[2] - fast_tanh(self.stage[3]));
        }
        self.stage[3]
    }
}

// --- AnalogOsc ---

#[derive(Clone, Debug)]
pub struct AnalogOsc {
    config: OscillatorConfig,
    phase: f32,
    nyquist: f32,
    phase_increment: f32,
    blep_table: Vec<f32>,
}

impl AnalogOsc {
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

    pub fn with_phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    fn advance_phase(&mut self, frequency: f32) {
        let frequency = frequency.min(self.nyquist);
        self.phase_increment = frequency / self.config.sample_rate as f32;
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }
    }

    pub fn generate(&mut self, wave_type: WaveType, frequency: f32) -> f32 {
        match wave_type {
            WaveType::Sine => self.generate_sine(frequency),
            WaveType::Square => self.generate_square(frequency, 0.5),
            WaveType::Sawtooth => self.generate_sawtooth(frequency),
            WaveType::Triangle => self.generate_triangle(frequency),
        }
    }

    pub fn generate_sine(&mut self, frequency: f32) -> f32 {
        let output = (2.0 * PI * self.phase).sin();
        self.advance_phase(frequency);
        output
    }

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

    pub fn generate_triangle(&mut self, frequency: f32) -> f32 {
        // Triangle from phase directly: linear ramps
        let output = if self.phase < 0.5 {
            4.0 * self.phase - 1.0
        } else {
            3.0 - 4.0 * self.phase
        };
        self.advance_phase(frequency);
        output
    }

    fn apply_blep(&mut self, output: &mut f32) {
        if self.phase < self.phase_increment {
            let index = (self.phase / self.phase_increment * self.blep_table.len() as f32) as usize;
            *output += self.blep_table.get(index).cloned().unwrap_or(0.0);
        }
    }
}

// --- Oscillator (public, implements rodio::Source) ---

pub struct Oscillator {
    freq: f32,
    wave_type: WaveType,
    voices: Vec<AnalogOsc>,
    voice_freq_multipliers: Vec<f32>,
    filter: Option<MoogLadderFilter>,
}

#[allow(dead_code)]
impl Oscillator {
    pub fn sine(freq: f32) -> Self {
        Self::new_single(freq, WaveType::Sine)
    }

    pub fn square(freq: f32) -> Self {
        Self::new_single(freq, WaveType::Square)
    }

    pub fn sawtooth(freq: f32) -> Self {
        Self::new_single(freq, WaveType::Sawtooth)
    }

    pub fn triangle(freq: f32) -> Self {
        Self::new_single(freq, WaveType::Triangle)
    }

    fn new_single(freq: f32, wave_type: WaveType) -> Self {
        let config = OscillatorConfig::default();
        Oscillator {
            freq,
            wave_type,
            voices: vec![AnalogOsc::new(config)],
            voice_freq_multipliers: vec![1.0],
            filter: None,
        }
    }

    /// Construct an oscillator from synth parameters.
    pub fn with_params(freq: f32, params: &SynthParams) -> Self {
        let config = OscillatorConfig::default();
        let n = params.unison_voices.max(1) as usize;

        let mut voices = Vec::with_capacity(n);
        let mut multipliers = Vec::with_capacity(n);

        for i in 0..n {
            // Spread voices symmetrically around center frequency
            let detune_cents = if n > 1 {
                let spread = params.detune_cents;
                let offset = i as f32 - (n - 1) as f32 / 2.0;
                let max_offset = (n - 1) as f32 / 2.0;
                if max_offset > 0.0 {
                    offset / max_offset * spread
                } else {
                    0.0
                }
            } else {
                0.0
            };
            let multiplier = 2.0_f32.powf(detune_cents / 1200.0);
            multipliers.push(multiplier);

            // Randomize initial phase per voice to avoid constructive interference
            let phase = if n > 1 {
                // Simple hash-based phase from voice index
                (i as f32 * 0.618033988) % 1.0
            } else {
                0.0
            };
            voices.push(AnalogOsc::new(config.clone()).with_phase(phase));
        }

        let filter = if params.filter_cutoff < 19900.0 {
            let mut f = MoogLadderFilter::new(config.sample_rate as f32);
            f.set_params(params.filter_cutoff, params.filter_resonance);
            Some(f)
        } else {
            None
        };

        Oscillator {
            freq,
            wave_type: params.wave_type,
            voices,
            voice_freq_multipliers: multipliers,
            filter,
        }
    }
}

impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let n = self.voices.len() as f32;
        let mut sample = 0.0;

        for (voice, &mult) in self.voices.iter_mut().zip(self.voice_freq_multipliers.iter()) {
            sample += voice.generate(self.wave_type, self.freq * mult);
        }
        sample /= n;

        if let Some(ref mut filter) = self.filter {
            sample = filter.process(sample);
        }

        Some(sample)
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
        48000
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}
