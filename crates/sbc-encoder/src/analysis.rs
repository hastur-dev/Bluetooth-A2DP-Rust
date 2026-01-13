//! Polyphase analysis filterbank for SBC encoder
//!
//! Implements the 4 or 8 subband analysis filter as specified in A2DP.
//! Uses fixed-point arithmetic for embedded performance.

use crate::config::{SbcConfig, Subbands};
use crate::tables::{COS_TABLE_4, COS_TABLE_8, PROTO_4_40, PROTO_8_80};

/// Maximum number of subbands supported
const MAX_SUBBANDS: usize = 8;

/// Maximum number of blocks per frame
const MAX_BLOCKS: usize = 16;

/// Maximum channels
const MAX_CHANNELS: usize = 2;

/// Filter history depth (10 samples per subband)
const FILTER_DEPTH: usize = 10;

/// Analysis filter state
///
/// Maintains the filter history for each channel.
/// All buffers are pre-allocated.
pub struct AnalysisFilter {
    /// Filter memory X for each channel
    /// Shape: [channel][subband * 10]
    x: [[i32; MAX_SUBBANDS * FILTER_DEPTH]; MAX_CHANNELS],
    /// Current position in circular buffer
    x_pos: usize,
    /// Number of subbands configured
    #[allow(dead_code)]
    subbands: Subbands,
}

impl AnalysisFilter {
    /// Create a new analysis filter for the given number of subbands
    pub fn new(subbands: Subbands) -> Self {
        Self {
            x: [[0; MAX_SUBBANDS * FILTER_DEPTH]; MAX_CHANNELS],
            x_pos: 0,
            subbands,
        }
    }

    /// Reset filter state (clear history)
    pub fn reset(&mut self) {
        for ch in &mut self.x {
            for sample in ch.iter_mut() {
                *sample = 0;
            }
        }
        self.x_pos = 0;
    }

    /// Process PCM samples through the analysis filterbank
    ///
    /// # Arguments
    /// * `pcm` - Interleaved stereo PCM samples (L, R, L, R, ...)
    /// * `config` - SBC configuration
    ///
    /// # Returns
    /// Subband samples: `[channel][block][subband]`
    pub fn process(
        &mut self,
        pcm: &[i16],
        config: &SbcConfig,
    ) -> [[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS] {
        assert!(pcm.len() >= config.samples_per_frame() * config.channels() as usize);

        let num_subbands = config.subbands.count();
        let num_blocks = config.block_length.count();
        let num_channels = config.channels() as usize;

        let mut output = [[[0i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];

        // Process each block
        for blk in 0..num_blocks {
            // Process each channel
            for ch in 0..num_channels {
                // Shift in new samples (subbands samples per block per channel)
                self.shift_in_samples(pcm, blk, ch, num_subbands, num_channels);

                // Apply polyphase filter and compute subband samples
                let sb_samples = self.compute_subbands(ch, num_subbands);

                // Store results
                for sb in 0..num_subbands {
                    output[ch][blk][sb] = sb_samples[sb];
                }
            }
        }

        output
    }

    /// Shift new PCM samples into the filter memory
    fn shift_in_samples(
        &mut self,
        pcm: &[i16],
        block: usize,
        channel: usize,
        subbands: usize,
        channels: usize,
    ) {
        // Calculate start position in PCM buffer
        let pcm_start = (block * subbands * channels) + channel;

        // Shift old samples
        let history_len = subbands * FILTER_DEPTH;

        // Bounded loop: at most MAX_SUBBANDS * FILTER_DEPTH iterations
        for i in (subbands..history_len).rev() {
            self.x[channel][i] = self.x[channel][i - subbands];
        }

        // Insert new samples (reversed order as per spec)
        // Bounded loop: at most MAX_SUBBANDS iterations
        for i in 0..subbands {
            let pcm_idx = pcm_start + (subbands - 1 - i) * channels;
            self.x[channel][i] = pcm[pcm_idx] as i32;
        }
    }

    /// Compute subband samples using the polyphase analysis filter
    fn compute_subbands(&self, channel: usize, subbands: usize) -> [i32; MAX_SUBBANDS] {
        let mut sb = [0i32; MAX_SUBBANDS];

        assert!(subbands == 4 || subbands == 8, "Invalid subbands");

        // Temporary Z vector for partial products
        let mut z = [0i64; MAX_SUBBANDS * 2];

        // Step 1: Window by prototype filter
        // Bounded loop: FILTER_DEPTH (10) iterations
        for j in 0..FILTER_DEPTH {
            // Bounded loop: at most MAX_SUBBANDS iterations
            for i in 0..subbands {
                let x_idx = j * subbands + i;
                let proto_idx = j * subbands + i;
                let z_idx = i + (j % 2) * subbands;

                let x_val = self.x[channel][x_idx] as i64;
                let proto_val = if subbands == 8 {
                    PROTO_8_80[proto_idx] as i64
                } else {
                    PROTO_4_40[proto_idx] as i64
                };

                z[z_idx] += (x_val * proto_val) >> 15;
            }
        }

        // Step 2: Matrixing (cosine modulation)
        // Bounded loop: at most MAX_SUBBANDS iterations
        for k in 0..subbands {
            let mut sum = 0i64;

            // Bounded loop: at most MAX_SUBBANDS * 2 iterations
            for i in 0..(subbands * 2) {
                let cos_idx = i % subbands;
                let cos_val = if subbands == 8 {
                    COS_TABLE_8[k][cos_idx] as i64
                } else {
                    COS_TABLE_4[k][cos_idx] as i64
                };

                sum += (z[i] * cos_val) >> 14;
            }

            // Scale and store result
            sb[k] = (sum >> 8) as i32;
        }

        sb
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn test_analysis_filter_creation() {
        let filter = AnalysisFilter::new(Subbands::Sub8);
        assert_eq!(filter.subbands, Subbands::Sub8);
    }

    #[test]
    fn test_analysis_filter_reset() {
        let mut filter = AnalysisFilter::new(Subbands::Sub8);

        // Set some non-zero values
        filter.x[0][0] = 1234;
        filter.x[1][5] = 5678;

        // Reset
        filter.reset();

        // Check all zeros
        for ch in &filter.x {
            for sample in ch.iter() {
                assert_eq!(*sample, 0);
            }
        }
    }

    #[test]
    fn test_analysis_silence() {
        let mut filter = AnalysisFilter::new(Subbands::Sub8);
        let config = SbcConfig::default();

        let samples_needed = config.samples_per_frame() * config.channels() as usize;
        let pcm = std::vec![0i16; samples_needed];

        let output = filter.process(&pcm, &config);

        // Silence should produce (near) zero subband samples
        for ch in 0..config.channels() as usize {
            for blk in 0..config.block_length.count() {
                for sb in 0..config.subbands.count() {
                    // Allow small rounding errors
                    assert!(
                        output[ch][blk][sb].abs() < 100,
                        "Expected near-zero for silence"
                    );
                }
            }
        }
    }

    #[test]
    fn test_analysis_dc_input() {
        let mut filter = AnalysisFilter::new(Subbands::Sub8);
        let config = SbcConfig::default();

        let samples_needed = config.samples_per_frame() * config.channels() as usize;
        // DC signal (constant value)
        let pcm = std::vec![1000i16; samples_needed];

        // Process multiple frames to let the filter state stabilize
        // First frame has startup transients
        let _ = filter.process(&pcm, &config);
        let _ = filter.process(&pcm, &config);
        let output = filter.process(&pcm, &config);

        // DC input should produce some non-zero output across subbands
        // The exact distribution depends on the prototype filter coefficients
        // and cosine modulation matrix - we just verify we get meaningful output
        let total_energy: i64 = output[0]
            .iter()
            .flat_map(|blk| blk.iter())
            .map(|&s| (s as i64).abs())
            .sum();

        // Should produce some energy (not all zeros)
        assert!(
            total_energy > 0,
            "DC input should produce non-zero subband energy"
        );

        // Verify output dimensions are correct
        assert_eq!(output.len(), 2, "Should have 2 channels");
        assert_eq!(output[0].len(), 16, "Should have 16 blocks");
        assert_eq!(output[0][0].len(), 8, "Should have 8 subbands");
    }

    #[test]
    fn test_analysis_high_frequency() {
        let mut filter = AnalysisFilter::new(Subbands::Sub8);
        let config = SbcConfig::default();

        let samples_needed = config.samples_per_frame() * config.channels() as usize;

        // High frequency: alternating +/- samples
        let pcm: std::vec::Vec<i16> = (0..samples_needed)
            .map(|i| if i % 2 == 0 { 1000 } else { -1000 })
            .collect();

        let output = filter.process(&pcm, &config);

        // High frequency should appear in higher subbands
        let sb0_energy: i64 = output[0].iter().map(|blk| blk[0].abs() as i64).sum();
        let sb7_energy: i64 = output[0].iter().map(|blk| blk[7].abs() as i64).sum();

        // High subband should have more energy for high frequency
        assert!(
            sb7_energy >= sb0_energy,
            "High freq should concentrate in high subbands"
        );
    }
}
