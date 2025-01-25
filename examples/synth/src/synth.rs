use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use rodio::source::Source;
use rodio::{OutputStreamHandle, Sink};

use crate::envelope::*;

/// Advanced synthesizer with improved envelope and audio management
pub struct Synth {
    audio_sources: HashMap<u8, (Sink, ADSREnvelope)>,
    stream_handle: OutputStreamHandle,
    default_envelope_config: ADSREnvelopeConfig,
}

impl Synth {
    /// Create a new AdvancedSynth with optional default envelope configuration
    pub fn new(
        stream_handle: OutputStreamHandle,
        default_envelope: Option<ADSREnvelopeConfig>,
    ) -> Self {
        Synth {
            audio_sources: HashMap::new(),
            stream_handle,
            default_envelope_config: default_envelope.unwrap_or_else(|| {
                ADSREnvelopeConfig::builder()
                    .attack(Duration::from_millis(50))
                    .decay(Duration::from_millis(100))
                    .sustain(0.7)
                    .release(Duration::from_millis(200))
                    .build()
                    .expect("Default envelope configuration failed")
            }),
        }
    }

    /// Play an audio source with optional custom envelope
    pub fn play_source(
        &mut self,
        audio_source: Box<dyn Source<Item = f32> + Send>,
        source_id: u8,
        envelope: Option<ADSREnvelopeConfig>,
    ) -> Result<(), String> {
        // Check if source ID is already in use
        if self.audio_sources.contains_key(&source_id) {
            return Err(format!("Source ID {} is already in use", source_id));
        }

        // Create a new sink
        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| format!("Failed to create audio sink: {}", e))?;

        // Append the audio source to the sink
        sink.append(audio_source);

        // Use provided envelope or default
        let envelope_config = envelope.unwrap_or_else(|| self.default_envelope_config.clone());
        let mut envelope = ADSREnvelope::new(envelope_config);

        // Start the envelope
        envelope.start();

        // Immediately set the initial volume
        let now = Instant::now();
        let initial_volume = envelope.get_amplitude(now);
        sink.set_volume(initial_volume);

        // Store the sink and envelope
        self.audio_sources.insert(source_id, (sink, envelope));

        Ok(())
    }

    // pub fn play_source(
    //     &mut self,
    //     audio_source: Box<dyn Source<Item = f32> + Send>,
    //     source_id: u8,
    //     envelope: Option<ADSREnvelopeConfig>,
    // ) -> Result<(), String> {
    //     // Remove existing source with the same ID to allow retriggering
    //     if self.audio_sources.contains_key(&source_id) {
    //         self.audio_sources.remove(&source_id);
    //     }

    //     // Proceed to create and add the new source
    //     let sink = Sink::try_new(&self.stream_handle)
    //         .map_err(|e| format!("Failed to create audio sink: {}", e))?;

    //     sink.append(audio_source);

    //     let envelope_config = envelope.unwrap_or_else(|| self.default_envelope_config.clone());
    //     let mut envelope = ADSREnvelope::new(envelope_config);
    //     envelope.start();

    //     let now = Instant::now();
    //     let initial_volume = envelope.get_amplitude(now);
    //     sink.set_volume(initial_volume);

    //     self.audio_sources.insert(source_id, (sink, envelope));

    //     Ok(())
    // }

    /// Release a specific audio source
    pub fn release_source(&mut self, source_id: u8) -> Result<(), String> {
        self.audio_sources
            .get_mut(&source_id)
            .map(|(_, envelope)| {
                envelope.release();
            })
            .ok_or_else(|| "Source not found".to_string())
    }

    /// Update audio sources and their envelopes
    pub fn update(&mut self) {
        if self.audio_sources.is_empty() {
            return;
        }

        let now = Instant::now();
        let mut sources_to_remove = Vec::new();

        // Update each source's volume based on its envelope
        for (source_id, (sink, envelope)) in self.audio_sources.iter_mut() {
            // Calculate current amplitude
            let volume = envelope.get_amplitude(now);
            sink.set_volume(volume);

            // Check if source is completed
            if envelope.is_finished(now) {
                sources_to_remove.push(*source_id);
            }
        }

        // Remove completed sources
        for source_id in sources_to_remove {
            self.audio_sources.remove(&source_id);
        }
    }
}

// Example usage and basic tests
#[cfg(test)]
mod tests {
    use super::*;
    use rodio::{OutputStream, Source};
    use std::collections::vec_deque::VecDeque;

    // A simple test source that generates a constant tone
    struct TestSource {
        samples: VecDeque<f32>,
    }

    impl TestSource {
        fn new(duration: usize) -> Self {
            Self {
                samples: std::iter::repeat(1.0).take(duration).collect(),
            }
        }
    }

    impl Iterator for TestSource {
        type Item = f32;
        fn next(&mut self) -> Option<Self::Item> {
            self.samples.pop_front()
        }
    }

    impl Source for TestSource {
        fn current_frame_len(&self) -> Option<usize> {
            Some(self.samples.len())
        }

        fn channels(&self) -> u16 {
            1
        }
        fn sample_rate(&self) -> u32 {
            44100
        }
        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }

    #[test]
    fn test_basic_synth_playback() {
        // Create an output stream
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();

        // Create a synth
        let mut synth = Synth::new(
            stream_handle,
            None, // Use default envelope
        );

        // Create a test audio source
        let test_source = TestSource::new(1000);

        // Play the source
        synth
            .play_source(
                Box::new(test_source),
                1, // source ID
                None,
            )
            .expect("Failed to play source");

        // Update a few times to simulate envelope progression
        for _ in 0..10 {
            synth.update();
        }
    }
}
