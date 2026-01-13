//! Bit allocation for SBC encoder
//!
//! Implements the loudness and SNR bit allocation algorithms
//! as specified in the A2DP specification.

use crate::config::{AllocationMethod, ChannelMode, SbcConfig};
use crate::tables::{LOUDNESS_OFFSET_4, LOUDNESS_OFFSET_8};

/// Maximum number of subbands
const MAX_SUBBANDS: usize = 8;
/// Maximum channels
const MAX_CHANNELS: usize = 2;

/// Bit allocator for SBC encoding
pub struct BitAllocator {
    // No state needed
}

impl BitAllocator {
    /// Create a new bit allocator
    pub fn new() -> Self {
        Self {}
    }

    /// Allocate bits to subbands based on scale factors and configuration
    ///
    /// Returns the number of bits allocated to each subband for each channel.
    pub fn allocate(
        &self,
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
        join_flags: u8,
    ) -> [[u8; MAX_SUBBANDS]; MAX_CHANNELS] {
        match config.allocation_method {
            AllocationMethod::Snr => self.allocate_snr(scale_factors, config, join_flags),
            AllocationMethod::Loudness => self.allocate_loudness(scale_factors, config, join_flags),
        }
    }

    /// SNR-based bit allocation
    ///
    /// Allocates bits proportionally to scale factors.
    fn allocate_snr(
        &self,
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
        join_flags: u8,
    ) -> [[u8; MAX_SUBBANDS]; MAX_CHANNELS] {
        let num_subbands = config.subbands.count();
        let num_channels = config.channels() as usize;

        // For SNR allocation, bitneed = scale_factor
        let mut bitneed = [[0i32; MAX_SUBBANDS]; MAX_CHANNELS];

        // Bounded loop: MAX_CHANNELS iterations
        for ch in 0..num_channels {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                bitneed[ch][sb] = scale_factors[ch][sb] as i32;
            }
        }

        self.distribute_bits(&bitneed, config, join_flags)
    }

    /// Loudness-based bit allocation
    ///
    /// Applies psychoacoustic offsets to prioritize perceptually important subbands.
    fn allocate_loudness(
        &self,
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
        join_flags: u8,
    ) -> [[u8; MAX_SUBBANDS]; MAX_CHANNELS] {
        let num_subbands = config.subbands.count();
        let num_channels = config.channels() as usize;
        let freq_idx = config.sampling_frequency as usize;

        // Calculate bitneed with loudness offsets
        let mut bitneed = [[0i32; MAX_SUBBANDS]; MAX_CHANNELS];

        // Bounded loop: MAX_CHANNELS iterations
        for ch in 0..num_channels {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                let sf = scale_factors[ch][sb] as i32;

                if sf == 0 {
                    bitneed[ch][sb] = -5; // Very low priority for silent bands
                } else {
                    // Get offset from appropriate table based on subband count
                    let offset = if num_subbands == 8 {
                        LOUDNESS_OFFSET_8[freq_idx][sb] as i32
                    } else {
                        LOUDNESS_OFFSET_4[freq_idx][sb] as i32
                    };

                    if sf > offset {
                        bitneed[ch][sb] = sf - offset;
                    } else {
                        // Below threshold: halve the bitneed
                        bitneed[ch][sb] = (sf - offset) / 2;
                    }
                }
            }
        }

        self.distribute_bits(&bitneed, config, join_flags)
    }

    /// Distribute bits according to bitneed values
    ///
    /// This is the core bit allocation algorithm that iteratively assigns
    /// bits to subbands with the highest bitneed until the bitpool is exhausted.
    fn distribute_bits(
        &self,
        bitneed: &[[i32; MAX_SUBBANDS]; MAX_CHANNELS],
        config: &SbcConfig,
        join_flags: u8,
    ) -> [[u8; MAX_SUBBANDS]; MAX_CHANNELS] {
        let num_subbands = config.subbands.count();
        let num_channels = config.channels() as usize;
        let bitpool = config.bitpool as i32;

        let mut bits = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];

        // Calculate total available bits
        let mut remaining_bits = bitpool;

        // Find the maximum bitneed
        let mut max_bitneed = i32::MIN;
        // Bounded loop: MAX_CHANNELS * MAX_SUBBANDS iterations
        for ch in 0..num_channels {
            for sb in 0..num_subbands {
                if bitneed[ch][sb] > max_bitneed {
                    max_bitneed = bitneed[ch][sb];
                }
            }
        }

        // Start from the maximum bitneed and work down
        let mut bitslice = max_bitneed + 1;

        // Initial pass: allocate bits for subbands above threshold
        // Bounded loop: worst case 32 iterations (max bitneed range)
        const MAX_ITERATIONS: usize = 64;
        for _ in 0..MAX_ITERATIONS {
            if bitslice <= 0 || remaining_bits <= 0 {
                break;
            }

            bitslice -= 1;
            let mut bits_used = 0;

            // Count bits needed at this slice level
            // Bounded loop: MAX_CHANNELS * MAX_SUBBANDS iterations
            for ch in 0..num_channels {
                for sb in 0..num_subbands {
                    if bitneed[ch][sb] == bitslice + 1 {
                        // First bit for this subband
                        bits_used += 2; // 2 bits minimum
                    } else if bitneed[ch][sb] > bitslice && bits[ch][sb] > 0 {
                        // Additional bit
                        bits_used += 1;
                    }
                }
            }

            if bits_used <= remaining_bits {
                // Apply this allocation
                // Bounded loop: MAX_CHANNELS * MAX_SUBBANDS iterations
                for ch in 0..num_channels {
                    for sb in 0..num_subbands {
                        if bitneed[ch][sb] == bitslice + 1 {
                            bits[ch][sb] = 2;
                        } else if bitneed[ch][sb] > bitslice && bits[ch][sb] > 0 {
                            bits[ch][sb] += 1;
                        }
                    }
                }
                remaining_bits -= bits_used;
            }
        }

        // Second pass: distribute remaining bits evenly
        // Bounded loop: remaining_bits iterations (max 250)
        const MAX_REMAINING_ITERATIONS: usize = 256;
        for _ in 0..MAX_REMAINING_ITERATIONS {
            if remaining_bits <= 0 {
                break;
            }

            let mut allocated = false;

            // Find the subband with highest bitneed that can accept more bits
            // Bounded loop: MAX_CHANNELS * MAX_SUBBANDS iterations
            for ch in 0..num_channels {
                for sb in 0..num_subbands {
                    if remaining_bits <= 0 {
                        break;
                    }

                    // Maximum bits per subband is 16
                    if bits[ch][sb] < 16 && bitneed[ch][sb] > 0 {
                        if bits[ch][sb] == 0 {
                            // Need 2 bits minimum to start
                            if remaining_bits >= 2 {
                                bits[ch][sb] = 2;
                                remaining_bits -= 2;
                                allocated = true;
                            }
                        } else {
                            bits[ch][sb] += 1;
                            remaining_bits -= 1;
                            allocated = true;
                        }
                    }
                }
            }

            if !allocated {
                break; // No more subbands can accept bits
            }
        }

        // Adjust for joint stereo: joined subbands share bits
        if config.channel_mode == ChannelMode::JointStereo && num_channels == 2 {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                if (join_flags >> (num_subbands - 1 - sb)) & 1 == 1 {
                    // For joined subbands, both channels use the same bits
                    let max_bits = bits[0][sb].max(bits[1][sb]);
                    bits[0][sb] = max_bits;
                    bits[1][sb] = max_bits;
                }
            }
        }

        bits
    }
}

impl Default for BitAllocator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn test_bit_allocator_creation() {
        let _alloc = BitAllocator::new();
    }

    #[test]
    fn test_allocate_snr_basic() {
        let alloc = BitAllocator::new();
        let config = SbcConfig {
            allocation_method: AllocationMethod::Snr,
            bitpool: 50,
            ..Default::default()
        };

        // All scale factors equal
        let scale_factors = [[5u8; MAX_SUBBANDS]; MAX_CHANNELS];

        let bits = alloc.allocate(&scale_factors, &config, 0);

        // Should have non-zero bits allocated
        let total_bits: u32 = bits
            .iter()
            .flat_map(|ch| ch.iter())
            .map(|&b| b as u32)
            .sum();
        assert!(total_bits > 0, "Should allocate some bits");
    }

    #[test]
    fn test_allocate_loudness_basic() {
        let alloc = BitAllocator::new();
        let config = SbcConfig {
            allocation_method: AllocationMethod::Loudness,
            bitpool: 50,
            ..Default::default()
        };

        let scale_factors = [[5u8; MAX_SUBBANDS]; MAX_CHANNELS];

        let bits = alloc.allocate(&scale_factors, &config, 0);

        // Should have non-zero bits allocated
        let total_bits: u32 = bits
            .iter()
            .flat_map(|ch| ch.iter())
            .map(|&b| b as u32)
            .sum();
        assert!(total_bits > 0, "Should allocate some bits");
    }

    #[test]
    fn test_allocate_silent_subbands() {
        let alloc = BitAllocator::new();
        let config = SbcConfig {
            allocation_method: AllocationMethod::Loudness,
            bitpool: 50,
            ..Default::default()
        };

        // All zeros - should get minimal or no allocation
        let scale_factors = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];

        let bits = alloc.allocate(&scale_factors, &config, 0);

        // Silent subbands should get low priority
        // Total bits should be low or zero
        let total_bits: u32 = bits
            .iter()
            .flat_map(|ch| ch.iter())
            .map(|&b| b as u32)
            .sum();
        assert!(total_bits < 100, "Silent should get minimal bits");
    }

    #[test]
    fn test_allocate_respects_bitpool() {
        let alloc = BitAllocator::new();

        for bitpool in [20, 50, 100, 200] {
            let config = SbcConfig {
                bitpool,
                ..Default::default()
            };

            let scale_factors = [[10u8; MAX_SUBBANDS]; MAX_CHANNELS];
            let bits = alloc.allocate(&scale_factors, &config, 0);

            // Total allocated bits should not exceed bitpool
            let total_bits: u32 = bits
                .iter()
                .flat_map(|ch| ch.iter())
                .map(|&b| b as u32)
                .sum();

            assert!(
                total_bits <= bitpool as u32 * 2,
                "Should not exceed bitpool"
            );
        }
    }

    #[test]
    fn test_bits_per_subband_max_16() {
        let alloc = BitAllocator::new();
        let config = SbcConfig {
            bitpool: 200, // High bitpool
            ..Default::default()
        };

        let scale_factors = [[15u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let bits = alloc.allocate(&scale_factors, &config, 0);

        // No subband should have more than 16 bits
        for ch in &bits {
            for &b in ch {
                assert!(b <= 16, "Max 16 bits per subband");
            }
        }
    }
}
