// This file is for the Oscillator struct, which implements the rodio source trait.

use rodio::source::Source;
use std::f32::consts::PI;

const SAMPLE_RATE: u32 = 48000; // The sample rate of the audio in Hz.

// The wave type of the oscillator
#[derive(Clone, Debug)]
enum WaveType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}

#[derive(Clone, Debug)]
pub struct Oscillator {
    freq: f32,
    num_sample: usize, // The number of samples that have been played
    wave_type: WaveType,
}

// Allow dead code is used because main.rs doesn't use all of the wave types, just one of them
// Without this, the compiler would complain about unused code.
impl Oscillator {
    #[allow(dead_code)]
    pub fn sine_wave(freq: f32) -> Oscillator {
        // Create a new sine wave oscillator
        Oscillator {
            freq,
            num_sample: 0,
            wave_type: WaveType::Sine,
        }
    }

    #[allow(dead_code)]
    pub fn square_wave(freq: f32) -> Oscillator {
        // Create a new square wave oscillator
        Oscillator {
            freq,
            num_sample: 0,
            wave_type: WaveType::Square,
        }
    }

    #[allow(dead_code)]
    pub fn sawtooth_wave(freq: f32) -> Oscillator {
        // Create a new sawtooth wave oscillator
        Oscillator {
            freq,
            num_sample: 0,
            wave_type: WaveType::Sawtooth,
        }
    }

    #[allow(dead_code)]
    pub fn triangle_wave(freq: f32) -> Oscillator {
        // Create a new triangle wave oscillator
        Oscillator {
            freq,
            num_sample: 0,
            wave_type: WaveType::Triangle,
        }
    }
}

// Rodio requires that Iterator is implemented
// The next function is called every time a new sample is needed
impl Iterator for Oscillator {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);

        let t = self.num_sample as f32 / SAMPLE_RATE as f32; // Time
        let value = 2.0 * PI * self.freq * t;

        match self.wave_type {
            WaveType::Sine => Some(value.sin()),            // Sine wave
            WaveType::Square => Some(value.sin().signum()), // Signing the sine wave locks it to 1 or -1, making it a square wave.
            WaveType::Sawtooth => Some(2.0 * (t % (1.0 / self.freq)) * self.freq - 1.0), // Sawtooth wave using modulo.
            WaveType::Triangle => Some(value.sin().asin()), // The arcsine of the sine wave makes it a triangle wave.
        }
    }
}

impl Source for Oscillator {
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> u16 {
        1 // Mono, not stereo
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None // Will continue indefinitely until stopped
    }
}
