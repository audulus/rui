use std::f32::consts::PI;

#[derive(Clone, Debug)]
pub struct AnalogOsc {
    sample_rate: usize,
    nyquist: f32,
    conv: f32,
    phase: f32,
    prev_sync: f32,
    future: [f32; FUTURE_SIZE],
    future_index: usize,
    prev_output: f32,
    blep_table: [f32; BLEP_SIZE],
}

const OVERSAMPLING: usize = 256;
const ZERO_CROSSINGS: usize = 16;
const TRANSITION_START: f32 = 8000.0;
const TRANSITION_END: f32 = 10000.0;
const FUTURE_SIZE: usize = ZERO_CROSSINGS * 2;
const BLEP_SIZE: usize = OVERSAMPLING * ZERO_CROSSINGS * 2 + 1;

impl AnalogOsc {
    pub fn new() -> AnalogOsc {
        let blep_table = {
            let mut table = [0.0; BLEP_SIZE];
            for i in 0..BLEP_SIZE {
                let x = (i as f32 - (BLEP_SIZE as f32 - 1.0) / 2.0) / (OVERSAMPLING as f32);
                table[i] = if x == 0.0 {
                    1.0
                } else {
                    // Raised cosine windowed sinc function
                    x.sin() / (PI * x) * (1.0 - (x / (ZERO_CROSSINGS as f32)).cos())
                };
            }
            table
        };

        AnalogOsc {
            sample_rate: 0,
            nyquist: 0.0,
            conv: 0.0,
            phase: 0.0,
            prev_sync: 0.0,
            future: [0.0; FUTURE_SIZE],
            future_index: 0,
            prev_output: 0.0,
            blep_table,
        }
    }

    pub fn set_sample_rate(&mut self, n: usize) {
        // Avoid a potential data race.
        if self.sample_rate != n {
            self.sample_rate = n;
            self.nyquist = n as f32 / 2.0;
            self.conv = 1.0 / self.sample_rate as f32;
        }
    }

    pub fn tick_saw(&mut self, f: f32, sync: f32, shape: f32) -> f32 {
        let f = f.min(self.nyquist);
        let shape = shape.clamp(0.0, 1.0);

        let rate = f * self.conv;

        let mut output = 0.0;

        // Advance phase.
        self.phase += rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Sync.
        if sync > 0.0 && self.prev_sync <= 0.0 {
            // Estimate how long ago zero-crossing happened
            // via linear interpolation and intersection
            // with x-axis.
            let frames_since = sync / (sync - self.prev_sync);

            // Reset phase.
            self.phase = frames_since * rate;
        }

        self.prev_sync = sync;

        // Render bleps.
        if f < TRANSITION_END {
            // Compute phase of the second saw.
            let mut aux_phase = self.phase + shape * 0.5;
            if aux_phase >= 1.0 {
                aux_phase -= 1.0;
            }

            // Naive dual saw.
            output = self.phase + aux_phase - 1.0;

            // If the function has decreased, add a blep.
            if output < self.prev_output && rate > 0.0 {
                let scale = self.prev_output - output + 2.0 * rate;
                self.add_blep(f32::min(self.phase, aux_phase) / rate, scale);
            }

            self.prev_output = output;

            // Correct with bleps.
            output += self.future[self.future_index];
            self.future[self.future_index] = 0.0;
            self.future_index = (self.future_index + 1) % FUTURE_SIZE;

            // Remove DC component based on frequency.
            // 5.473 is emperically determined.
            output -= (f / self.sample_rate as f32) * 5.473;
        }

        // Render sine.
        if f > TRANSITION_START {
            let sine_gain = if f < TRANSITION_END {
                (f - TRANSITION_START) / (TRANSITION_END - TRANSITION_START)
            } else {
                1.0
            };

            output = (1.0 - sine_gain) * output + sine_gain * (f32::sin(self.phase * 2.0 * PI));
        }

        output
    }

    #[allow(dead_code)]
    pub fn process_saw(&mut self, pitch: &[f32], sync: &[f32], shape: &[f32], out: &mut [f32]) {
        let n = pitch.len();
        for i in 0..n {
            out[i] = self.tick_saw(pitch[i], sync[i], shape[i]);
        }
    }

    pub fn tick_square(&mut self, f: f32, sync: f32, shape: f32) -> f32 {
        let f = f.min(self.nyquist);
        let shape = shape.min(1.0).max(0.0);

        let rate = f * self.conv;

        let mut output = 0.0;

        // Pulse width.
        let pw = 0.5 * (1.0 - shape);

        // Advance phase.
        self.phase += rate;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        // Sync.
        if sync > 0.0 && self.prev_sync <= 0.0 {
            // Estimate how long ago zero-crossing happened
            // via linear interpolation and intersection
            // with x-axis.
            let frames_since = sync / (sync - self.prev_sync);

            // Reset phase.
            self.phase = frames_since * rate;
        }

        self.prev_sync = sync;

        // Render bleps.
        if f < TRANSITION_END {
            // Naive square.
            output = if self.phase < pw { 1.0 } else { -1.0 };

            if output != self.prev_output && rate > 0.0 {
                if self.phase < pw {
                    self.add_blep(self.phase / rate, -2.0);
                } else {
                    self.add_blep((self.phase - pw) / rate, 2.0);
                }
            }

            self.prev_output = output;

            // Correct with bleps.
            output += self.future[self.future_index];
            self.future[self.future_index] = 0.0;
            self.future_index = (self.future_index + 1) % FUTURE_SIZE;
        }

        // Render sine.
        if f > TRANSITION_START {
            let sine_gain = if f < TRANSITION_END {
                (f - TRANSITION_START) / (TRANSITION_END - TRANSITION_START)
            } else {
                1.0
            };

            output = (1.0 - sine_gain) * output + sine_gain * (f32::sin(self.phase * 2.0 * PI));
        }

        output
    }

    #[allow(dead_code)]
    pub fn process_square(&mut self, pitch: &[f32], sync: &[f32], shape: &[f32], out: &mut [f32]) {
        let n = pitch.len();
        for i in 0..n {
            out[i] = self.tick_square(pitch[i], sync[i], shape[i]);
        }
    }

    fn add_blep(&mut self, phase: f32, scale: f32) {
        // Add a blep into the future buffer.

        let mut p = (phase * (OVERSAMPLING as f32)) as usize; // Convert to integer index outside the loop.

        let mut i = self.future_index;

        // Note: should be able to do one loop with modulo. Perhaps that was slower?

        while i < FUTURE_SIZE && p < BLEP_SIZE {
            self.future[i] += self.blep_table[p] * scale;
            p += OVERSAMPLING;
            i += 1;
        }

        i = 0;
        while i < self.future_index && p < BLEP_SIZE {
            self.future[i] += self.blep_table[p] * scale;
            p += OVERSAMPLING;
            i += 1;
        }
    }
}
