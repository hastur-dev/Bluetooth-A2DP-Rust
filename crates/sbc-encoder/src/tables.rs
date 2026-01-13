//! Pre-computed tables for SBC encoder
//!
//! All values are in fixed-point format.
//! These are stored in ROM/Flash, not RAM.

/// Prototype filter coefficients for 8-subband analysis
/// 80 coefficients (10 per subband), Q15 format
/// From Bluetooth A2DP specification, Table 12.8
pub const PROTO_8_80: [i32; 80] = [
    0x0000_0000,
    0x0000_0083,
    -2877,  // 0xFFFF_F4C3 as signed
    0x0000_1649,
    -9735,  // 0xFFFF_D9F9
    0x0000_61EC,
    -36987, // 0xFFFF_6F85
    0x0001_A8B6,
    -212481, // 0xFFFC_C1FF
    0x000A_C911,
    // Window 1
    0x0000_0001,
    -127,   // 0xFFFF_FF81
    0x0000_0B67,
    -5704,  // 0xFFFF_E9B8
    0x0000_267A,
    -15248, // 0xFFFF_C470
    0x0000_9F97,
    -107119, // 0xFFFE_5D91
    0x0003_8083,
    -664312, // 0xFFF5_DD08
    // Window 2
    0x0000_0002,
    -238,   // 0xFFFF_FF12
    0x0000_05A0,
    -3217,  // 0xFFFF_F36F
    0x0000_0C9D,
    -6530,  // 0xFFFF_E67E
    0x0000_3F27,
    -29167, // 0xFFFF_8E11
    0x0000_DF5D,
    -203322, // 0xFFFC_E5C6
    // Window 3
    0x0000_0001,
    -26,    // 0xFFFF_FFE6
    0x0000_0110,
    -955,   // 0xFFFF_FC45
    -15,    // 0xFFFF_FFF1
    -1322,  // 0xFFFF_FAD6
    -1722,  // 0xFFFF_F946
    0x0000_0336,
    -10336, // 0xFFFF_D7A0
    0x0000_2C05,
    // Window 4
    0x0000_0000,
    0x0000_0000,
    -15,    // 0xFFFF_FFF1
    0x0000_0030,
    -166,   // 0xFFFF_FF5A
    0x0000_015D,
    -1252,  // 0xFFFF_FB1C
    0x0000_0951,
    -7316,  // 0xFFFF_E36C
    0x0000_46E6,
    // Window 5
    0x0000_0000,
    0x0000_0001,
    -11,    // 0xFFFF_FFF5
    0x0000_002B,
    -130,   // 0xFFFF_FF7E
    0x0000_00D8,
    -417,   // 0xFFFF_FE5F
    0x0000_03A9,
    -2481,  // 0xFFFF_F64F
    0x0000_14F2,
    // Window 6
    0x0000_0000,
    0x0000_0000,
    -3,     // 0xFFFF_FFFD
    0x0000_000A,
    -37,    // 0xFFFF_FFDB
    0x0000_002D,
    -82,    // 0xFFFF_FFAE
    0x0000_0069,
    -147,   // 0xFFFF_FF6D
    0x0000_0099,
    // Window 7
    0x0000_0000,
    0x0000_0000,
    0x0000_0000,
    0x0000_0001,
    -4,     // 0xFFFF_FFFC
    0x0000_0006,
    -7,     // 0xFFFF_FFF9
    0x0000_0009,
    -3,     // 0xFFFF_FFFD
    0x0000_0003,
];

/// Prototype filter coefficients for 4-subband analysis
/// 40 coefficients (10 per subband), Q15 format
pub const PROTO_4_40: [i32; 40] = [
    0x0000_0000,
    0x0000_0166,
    -5779,   // 0xFFFF_E96D
    0x0000_2C95,
    -19470,  // 0xFFFF_B3F2
    0x0000_C3D9,
    -73976,  // 0xFFFE_DF08
    0x0003_5142,
    -424964, // 0xFFF9_83FC
    0x0015_9222,
    // Window 1
    0x0000_0002,
    -253,    // 0xFFFF_FF03
    0x0000_16B4,
    -11408,  // 0xFFFF_D370
    0x0000_4CD5,
    -30496,  // 0xFFFF_88E0
    0x0001_3F4F,
    -214238, // 0xFFFC_BB22
    0x0007_0107,
    -1328624, // 0xFFEB_BA10
    // Window 2
    0x0000_0000,
    0x0000_0000,
    -15,     // 0xFFFF_FFF1
    0x0000_0061,
    -332,    // 0xFFFF_FEB4
    0x0000_02BA,
    -2504,   // 0xFFFF_F638
    0x0000_12A2,
    -14631,  // 0xFFFF_C6D9
    0x0000_8DCC,
    // Window 3
    0x0000_0000,
    0x0000_0000,
    -3,      // 0xFFFF_FFFD
    0x0000_0009,
    -43,     // 0xFFFF_FFD5
    0x0000_003B,
    -104,    // 0xFFFF_FF98
    0x0000_007A,
    -67,     // 0xFFFF_FFBD
    0x0000_0046,
];

/// Cosine modulation matrix for 8-subband analysis (M8)
/// cos(pi/8 * (k+0.5) * (2*n+5)) for k=0..7, n=0..7
/// Q14 format
pub const COS_TABLE_8: [[i32; 8]; 8] = [
    [0x2D41, 0x2D41, 0x2D41, 0x2D41, 0x2D41, 0x2D41, 0x2D41, 0x2D41],
    [0x3B21, 0x3B21, 0x187E, -0x187E, -0x3B21, -0x3B21, -0x187E, 0x187E],
    [0x3B21, 0x0000, -0x3B21, -0x3B21, 0x0000, 0x3B21, 0x3B21, 0x0000],
    [0x3B21, -0x187E, -0x3B21, 0x187E, 0x3B21, -0x187E, -0x3B21, 0x187E],
    [0x2D41, -0x2D41, -0x2D41, 0x2D41, 0x2D41, -0x2D41, -0x2D41, 0x2D41],
    [0x187E, -0x3B21, 0x187E, 0x187E, -0x3B21, 0x187E, 0x187E, -0x3B21],
    [0x0000, -0x3B21, 0x3B21, 0x0000, -0x3B21, 0x3B21, 0x0000, -0x3B21],
    [-0x187E, -0x187E, 0x3B21, -0x3B21, 0x187E, 0x187E, -0x3B21, 0x3B21],
];

/// Cosine modulation matrix for 4-subband analysis (M4)
/// Q14 format
pub const COS_TABLE_4: [[i32; 4]; 4] = [
    [0x2D41, 0x2D41, 0x2D41, 0x2D41],
    [0x3B21, 0x187E, -0x187E, -0x3B21],
    [0x2D41, -0x2D41, -0x2D41, 0x2D41],
    [0x187E, -0x3B21, 0x3B21, -0x187E],
];

/// Loudness offset table for 8 subbands at different sampling frequencies
/// [freq_index][subband]
pub const LOUDNESS_OFFSET_8: [[i8; 8]; 4] = [
    // 16 kHz
    [-1, 0, 0, 0, 0, 0, 0, 1],
    // 32 kHz
    [-2, 0, 0, 0, 0, 0, 1, 2],
    // 44.1 kHz
    [-2, 0, 0, 0, 0, 0, 1, 2],
    // 48 kHz
    [-2, 0, 0, 0, 0, 0, 1, 2],
];

/// Loudness offset table for 4 subbands
pub const LOUDNESS_OFFSET_4: [[i8; 4]; 4] = [
    // 16 kHz
    [-1, 0, 0, 1],
    // 32 kHz
    [-2, 0, 0, 2],
    // 44.1 kHz
    [-2, 0, 0, 2],
    // 48 kHz
    [-2, 0, 0, 2],
];

/// Power-of-two table for scale factor decoding
/// 2^(scale_factor + 1) for quantization
pub const SCALE_FACTOR_LEVELS: [i32; 16] = [
    2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_proto_8_80_length() {
        assert_eq!(PROTO_8_80.len(), 80);
    }

    #[test]
    fn test_cos_table_8_dimensions() {
        assert_eq!(COS_TABLE_8.len(), 8);
        assert_eq!(COS_TABLE_8[0].len(), 8);
    }

    #[test]
    fn test_scale_factor_levels() {
        assert_eq!(SCALE_FACTOR_LEVELS[0], 2);
        assert_eq!(SCALE_FACTOR_LEVELS[15], 65536);
    }
}
