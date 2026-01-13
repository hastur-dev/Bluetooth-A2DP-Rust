//! SBC frame packing
//!
//! Packs encoded subband samples into the SBC frame format
//! as specified in the A2DP specification.

use crate::config::{ChannelMode, SbcConfig};

/// Maximum subbands
const MAX_SUBBANDS: usize = 8;
/// Maximum blocks
const MAX_BLOCKS: usize = 16;
/// Maximum channels
const MAX_CHANNELS: usize = 2;

/// SBC sync word
const SBC_SYNCWORD: u8 = 0x9C;

/// Frame packer for SBC encoding
pub struct FramePacker {
    // Bit buffer for packing
    bit_buffer: u32,
    bits_in_buffer: u8,
}

impl FramePacker {
    /// Create a new frame packer
    pub fn new() -> Self {
        Self {
            bit_buffer: 0,
            bits_in_buffer: 0,
        }
    }

    /// Reset the bit buffer
    fn reset(&mut self) {
        self.bit_buffer = 0;
        self.bits_in_buffer = 0;
    }

    /// Write bits to the output buffer
    fn write_bits(&mut self, output: &mut [u8], pos: &mut usize, value: u32, num_bits: u8) {
        assert!(num_bits <= 32, "Too many bits");
        assert!(num_bits > 0, "Zero bits");

        // Add new bits to buffer
        self.bit_buffer = (self.bit_buffer << num_bits) | (value & ((1 << num_bits) - 1));
        self.bits_in_buffer += num_bits;

        // Flush complete bytes
        // Bounded loop: at most 4 iterations (32 bits / 8)
        for _ in 0..4 {
            if self.bits_in_buffer < 8 {
                break;
            }

            self.bits_in_buffer -= 8;
            let byte = ((self.bit_buffer >> self.bits_in_buffer) & 0xFF) as u8;

            assert!(*pos < output.len(), "Output buffer overflow");
            output[*pos] = byte;
            *pos += 1;
        }
    }

    /// Flush remaining bits (with zero padding)
    fn flush(&mut self, output: &mut [u8], pos: &mut usize) {
        if self.bits_in_buffer > 0 {
            // Pad with zeros
            let padding = 8 - self.bits_in_buffer;
            let byte = ((self.bit_buffer << padding) & 0xFF) as u8;

            assert!(*pos < output.len(), "Output buffer overflow");
            output[*pos] = byte;
            *pos += 1;
        }

        self.reset();
    }

    /// Pack an SBC frame
    ///
    /// # Arguments
    /// * `config` - SBC configuration
    /// * `join_flags` - Joint stereo flags (one bit per subband)
    /// * `scale_factors` - Scale factors for each channel/subband
    /// * `bits` - Bits allocated to each channel/subband
    /// * `samples` - Quantized samples
    /// * `output` - Output buffer
    ///
    /// # Returns
    /// Number of bytes written
    pub fn pack(
        &mut self,
        config: &SbcConfig,
        join_flags: u8,
        scale_factors: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        bits: &[[u8; MAX_SUBBANDS]; MAX_CHANNELS],
        samples: &[[[u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS],
        output: &mut [u8],
    ) -> usize {
        assert!(output.len() >= 4, "Output buffer too small for header");

        self.reset();
        let mut pos = 0;

        // --- Header (4 bytes) ---

        // Byte 0: Sync word
        output[pos] = SBC_SYNCWORD;
        pos += 1;

        // Byte 1: Sampling frequency (2) + Blocks (2) + Channel mode (2) + Alloc (1) + Subbands (1)
        let byte1 = (config.sampling_frequency.header_bits() << 6)
            | (config.block_length.header_bits() << 4)
            | (config.channel_mode.header_bits() << 2)
            | (config.allocation_method.header_bits() << 1)
            | config.subbands.header_bits();
        output[pos] = byte1;
        pos += 1;

        // Byte 2: Bitpool
        output[pos] = config.bitpool;
        pos += 1;

        // Byte 3: CRC (calculated later, placeholder for now)
        let crc_pos = pos;
        output[pos] = 0;
        pos += 1;

        let num_subbands = config.subbands.count();
        let num_blocks = config.block_length.count();
        let num_channels = config.channels() as usize;

        // --- Joint stereo flags (if applicable) ---
        if config.channel_mode == ChannelMode::JointStereo {
            self.write_bits(output, &mut pos, join_flags as u32, num_subbands as u8);
        }

        // --- Scale factors ---
        // Bounded loop: MAX_CHANNELS iterations
        for ch in 0..num_channels {
            // Bounded loop: MAX_SUBBANDS iterations
            for sb in 0..num_subbands {
                self.write_bits(output, &mut pos, scale_factors[ch][sb] as u32, 4);
            }
        }

        // --- Audio samples ---
        // Bounded loop: MAX_BLOCKS iterations
        for blk in 0..num_blocks {
            // Bounded loop: MAX_CHANNELS iterations
            for ch in 0..num_channels {
                // Bounded loop: MAX_SUBBANDS iterations
                for sb in 0..num_subbands {
                    let bit_count = bits[ch][sb];
                    if bit_count > 0 {
                        self.write_bits(output, &mut pos, samples[ch][blk][sb] as u32, bit_count);
                    }
                }
            }
        }

        // Flush remaining bits
        self.flush(output, &mut pos);

        // Calculate and write CRC
        output[crc_pos] = self.calc_crc(&output[0..pos]);

        pos
    }

    /// Calculate CRC-8 for the frame
    ///
    /// CRC covers bytes 1-3 and the scale factor + sample data
    fn calc_crc(&self, data: &[u8]) -> u8 {
        // CRC-8 polynomial: x^8 + x^4 + x^3 + x^2 + 1 = 0x1D
        const CRC_POLY: u8 = 0x1D;

        let mut crc: u8 = 0x0F; // Initial value

        // CRC over header bytes 1-2 (skip sync word at byte 0, CRC at byte 3)
        // Bounded loop: data.len() iterations (max ~512 for largest frame)
        for i in 1..data.len() {
            if i == 3 {
                continue; // Skip CRC byte itself
            }

            let byte = data[i];

            // Bounded loop: 8 iterations
            for bit in 0..8 {
                let msb = (crc >> 7) & 1;
                crc <<= 1;

                if ((byte >> (7 - bit)) & 1) ^ msb == 1 {
                    crc ^= CRC_POLY;
                }
            }
        }

        crc
    }
}

impl Default for FramePacker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::*;

    #[test]
    fn test_frame_packer_creation() {
        let _packer = FramePacker::new();
    }

    #[test]
    fn test_pack_header() {
        let mut packer = FramePacker::new();
        let config = SbcConfig::default();

        let scale_factors = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let bits = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let samples = [[[0u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];

        let mut output = [0u8; 512];
        let size = packer.pack(&config, 0, &scale_factors, &bits, &samples, &mut output);

        // Check sync word
        assert_eq!(output[0], SBC_SYNCWORD, "Sync word should be 0x9C");

        // Check bitpool
        assert_eq!(output[2], config.bitpool, "Bitpool should match config");

        // Size should be reasonable
        assert!(size >= 4 && size <= 512, "Frame size should be reasonable");
    }

    #[test]
    fn test_pack_with_data() {
        let mut packer = FramePacker::new();
        let config = SbcConfig::default();

        // Non-zero scale factors and bits
        let scale_factors = [[5u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let mut bits = [[0u8; MAX_SUBBANDS]; MAX_CHANNELS];
        bits[0][0] = 4;
        bits[0][1] = 4;
        bits[1][0] = 4;
        bits[1][1] = 4;

        // Some sample data
        let mut samples = [[[0u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];
        for blk in 0..16 {
            samples[0][blk][0] = 7;
            samples[0][blk][1] = 8;
            samples[1][blk][0] = 7;
            samples[1][blk][1] = 8;
        }

        let mut output = [0u8; 512];
        let size = packer.pack(&config, 0, &scale_factors, &bits, &samples, &mut output);

        assert!(size > 4, "Should have data beyond header");
        assert_eq!(output[0], SBC_SYNCWORD);
    }

    #[test]
    fn test_pack_joint_stereo() {
        let mut packer = FramePacker::new();
        let config = SbcConfig {
            channel_mode: ChannelMode::JointStereo,
            ..Default::default()
        };

        let scale_factors = [[5u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let bits = [[2u8; MAX_SUBBANDS]; MAX_CHANNELS];
        let samples = [[[1u16; MAX_SUBBANDS]; MAX_BLOCKS]; MAX_CHANNELS];

        let mut output = [0u8; 512];
        let join_flags = 0b11111110; // All but last subband joined

        let size = packer.pack(
            &config,
            join_flags,
            &scale_factors,
            &bits,
            &samples,
            &mut output,
        );

        assert!(size > 4);
        assert_eq!(output[0], SBC_SYNCWORD);
    }

    #[test]
    fn test_crc_calculation() {
        let packer = FramePacker::new();

        // Simple test data
        let data = [SBC_SYNCWORD, 0x35, 0x35, 0x00, 0x00, 0x00, 0x00, 0x00];

        let crc = packer.calc_crc(&data);

        // CRC should be non-zero for non-trivial data
        // Exact value depends on the polynomial and initial value
        assert!(crc != 0 || data[1..3].iter().all(|&x| x == 0));
    }
}
