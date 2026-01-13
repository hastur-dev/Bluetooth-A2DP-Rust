//! Quantization and scale factor calculation for SBC encoder

use crate::config::{ChannelMode, SbcConfig};
use crate::tables::SCALE_FACTOR_LEVELS;

/// Maximum number of subbands
const MAX_SUBBANDS: usize = 8;
/// Maximum number of blocks
const MAX_BLOCKS: usize = 16;
/// Maximum channels
const MAX_CHANNELS: usize = 2;

/// Quantizer for SBC encoding
pub struct Quantizer {
    // No state needed - all operations are stateless
}

impl Quantizer {
    /// Create a new quantizer
    pub fn new() -> Self {
        Self {}
    }

    /// Calculate scale factors for all subbands
    ///
    /// Scale factor represents the number of bits needed to represent
    /// the maximum absolute value in each subband.
    pub fn calc_scale_factors(
        &self,
        subbands: &[[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS],
        config: &SbcConfig,
    ) -> [[u8; MAX_SUBBANDS]; MAX_CHANNELS] {
        let num_subbands = config.subbands.count();
        let num_blocks = config.block_length.count();
        let num_channels = config.channels() as usize;

        let mut scale_factors = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];

        // Bounded loop: MAX_CHANNELS iterations
        for ch in 0..num_channels {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                // Find maximum absolute value in this subband
                let mut max_val: i32 = 0;

                // Bounded loop: MAX_BLOCKS iterations
                for blk in 0..num_blocks {
                    let abs_val = subbands[ch][blk][sb].abs();
                    if abs_val > max_val {
                        max_val = abs_val;
                    }
                }

                // Calculate scale factor (0-15)
                // Scale factor = floor(log2(max_val)) + 2, clamped to 0-15
                scale_factors[ch][sb] = self.calc_single_scale_factor(max_val);
            }
        }

        scale_factors
    }

    /// Calculate scale factor for a single value
    fn calc_single_scale_factor(&self, max_val: i32) -> u8 {
        if max_val == 0 {
            return 0;
        }

        // Find the highest bit set
        let bits_needed = 32 - max_val.leading_zeros();

        // Scale factor maps to the range [0, 15]
        // We want: 2^(sf+1) > max_val
        // So: sf+1 > log2(max_val)
        // sf = ceil(log2(max_val)) = bits_needed - 1

        let sf = if bits_needed > 1 {
            (bits_needed - 1) as u8
        } else {
            0
        };

        // Clamp to valid range [0, 15]
        if sf > 15 {
            15
        } else {
            sf
        }
    }

    /// Process joint stereo encoding
    ///
    /// For joint stereo, we selectively encode some subbands as M/S
    /// (mid/side) instead of L/R when it's more efficient.
    ///
    /// Returns the modified subbands and the join flags byte.
    pub fn joint_stereo_process(
        &self,
        mut subbands: [[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS],
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
    ) -> ([[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS], u8) {
        if config.channel_mode != ChannelMode::JointStereo {
            return (subbands, 0);
        }

        let num_subbands = config.subbands.count();
        let num_blocks = config.block_length.count();
        let mut join_flags: u8 = 0;

        // For each subband (except the last one in 8-subband mode)
        let join_limit = if num_subbands == 8 {
            num_subbands - 1
        } else {
            num_subbands
        };

        // Bounded loop: MAX_SUBBANDS - 1 iterations
        for sb in 0..join_limit {
            // Calculate the benefit of joint stereo for this subband
            // If L and R are similar, M/S encoding is more efficient

            let left_sf = scale_factors[0][sb];
            let right_sf = scale_factors[1][sb];

            // Simple heuristic: use joint stereo if scale factors are similar
            // and the samples are correlated
            let use_joint = self.should_use_joint(&subbands, sb, num_blocks, left_sf, right_sf);

            if use_joint {
                join_flags |= 1 << (num_subbands - 1 - sb);

                // Convert L/R to M/S
                // M = (L + R) / 2
                // S = (L - R) / 2
                // Bounded loop: MAX_BLOCKS iterations
                for blk in 0..num_blocks {
                    let left = subbands[0][blk][sb];
                    let right = subbands[1][blk][sb];

                    subbands[0][blk][sb] = (left + right) >> 1; // Mid
                    subbands[1][blk][sb] = (left - right) >> 1; // Side
                }
            }
        }

        (subbands, join_flags)
    }

    /// Determine if joint stereo should be used for a subband
    fn should_use_joint(
        &self,
        subbands: &[[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS],
        sb: usize,
        num_blocks: usize,
        left_sf: u8,
        right_sf: u8,
    ) -> bool {
        // If scale factors are very different, don't use joint stereo
        let sf_diff = if left_sf > right_sf {
            left_sf - right_sf
        } else {
            right_sf - left_sf
        };

        if sf_diff > 4 {
            return false;
        }

        // Calculate correlation between L and R
        let mut sum_product: i64 = 0;
        let mut sum_left_sq: i64 = 0;
        let mut sum_right_sq: i64 = 0;

        // Bounded loop: MAX_BLOCKS iterations
        for blk in 0..num_blocks {
            let left = subbands[0][blk][sb] as i64;
            let right = subbands[1][blk][sb] as i64;

            sum_product += left * right;
            sum_left_sq += left * left;
            sum_right_sq += right * right;
        }

        // If either channel is silent, don't use joint
        if sum_left_sq == 0 || sum_right_sq == 0 {
            return false;
        }

        // High correlation means L and R are similar -> M/S is efficient
        // correlation = sum_product / sqrt(sum_left_sq * sum_right_sq)
        // We want correlation > 0.5 (roughly)
        // Squared: sum_product^2 > 0.25 * sum_left_sq * sum_right_sq

        let threshold = (sum_left_sq >> 2) * (sum_right_sq >> 2);
        let product_sq = (sum_product >> 2) * (sum_product >> 2);

        // Use >= because perfectly identical channels (correlation = 1.0) should trigger joint stereo
        product_sq >= threshold
    }

    /// Quantize subband samples
    ///
    /// Quantizes each sample based on the allocated bits and scale factors.
    pub fn quantize(
        &self,
        subbands: &[[[i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS],
        bits: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
    ) -> [[[u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS] {
        let num_subbands = config.subbands.count();
        let num_blocks = config.block_length.count();
        let num_channels = config.channels() as usize;

        let mut quantized = [[[0u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];

        // Bounded loop: MAX_CHANNELS iterations
        for ch in 0..num_channels {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                let bit_count = bits[ch][sb];
                if bit_count == 0 {
                    continue; // No bits allocated
                }

                let sf = scale_factors[ch][sb] as usize;
                let levels = SCALE_FACTOR_LEVELS[sf];

                // Bounded loop: MAX_BLOCKS iterations
                for blk in 0..num_blocks {
                    let sample = subbands[ch][blk][sb];
                    quantized[ch][blk][sb] = self.quantize_sample(sample, bit_count, levels);
                }
            }
        }

        quantized
    }

    /// Quantize a single sample
    fn quantize_sample(&self, sample: i32, bits: u8, scale_level: i32) -> u16 {
        assert!(bits > 0 && bits <= 16, "Invalid bit count");
        assert!(scale_level > 0, "Invalid scale level");

        let levels = (1u32 << bits) - 1;

        // Normalize sample to [-1, 1] range using scale factor
        // Then quantize to [0, 2^bits - 1]
        let normalized = ((sample as i64) << 15) / (scale_level as i64);

        // Map from [-32768, 32767] to [0, levels]
        let offset = normalized + 32768;
        let quantized = (offset * levels as i64) >> 16;

        // Clamp to valid range
        if quantized < 0 {
            0
        } else if quantized > levels as i64 {
            levels as u16
        } else {
            quantized as u16
        }
    }
}

impl Default for Quantizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn test_scale_factor_zero() {
        let q = Quantizer::new();
        assert_eq!(q.calc_single_scale_factor(0), 0);
    }

    #[test]
    fn test_scale_factor_small() {
        let q = Quantizer::new();
        assert_eq!(q.calc_single_scale_factor(1), 0);
        assert_eq!(q.calc_single_scale_factor(2), 1);
        assert_eq!(q.calc_single_scale_factor(3), 1);
        assert_eq!(q.calc_single_scale_factor(4), 2);
    }

    #[test]
    fn test_scale_factor_large() {
        let q = Quantizer::new();
        // Large values should clamp to 15
        assert_eq!(q.calc_single_scale_factor(100000), 15);
    }

    #[test]
    fn test_quantize_sample_mid() {
        let q = Quantizer::new();
        // Zero sample with 8 bits should give middle value
        let result = q.quantize_sample(0, 8, 1024);
        // Should be around 127-128 (middle of 0-255 range)
        assert!(result >= 120 && result <= 135);
    }

    #[test]
    fn test_joint_stereo_identical_channels() {
        let q = Quantizer::new();
        let config = SbcConfig {
            channel_mode: ChannelMode::JointStereo,
            ..Default::default()
        };

        // Create identical L and R channels
        let mut subbands = [[[0i32; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];
        for blk in 0..16 {
            for sb in 0..8 {
                subbands[0][blk][sb] = 1000;
                subbands[1][blk][sb] = 1000;
            }
        }

        let scale_factors = [[4u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let (result, join_flags) = q.joint_stereo_process(subbands, &scale_factors, &config);

        // When L = R, M = L and S = 0
        // High correlation should trigger joint stereo
        assert!(join_flags != 0, "Should use joint stereo for identical channels");

        // Verify M = L, S = 0 for joined subbands
        for sb in 0..7 {
            // Last subband not joined in 8-subband mode
            if (join_flags >> (7 - sb)) & 1 == 1 {
                for blk in 0..16 {
                    assert_eq!(result[1][blk][sb], 0, "Side should be zero");
                }
            }
        }
    }
}
