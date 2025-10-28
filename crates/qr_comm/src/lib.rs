use qrcode::{EcLevel, Version};

pub mod data_structures;
pub mod matrix;
pub mod network_coding;

pub const MAX_PARTICIPANTS: usize = 16;
pub const MAX_PARTICIPANTS_SIZE_BITS: usize = MAX_PARTICIPANTS.next_power_of_two().ilog2() as usize;
pub const MAX_PARTICIPANTS_SIZE_BYTES: usize = MAX_PARTICIPANTS_SIZE_BITS.div_ceil(8);

pub const HEADER_SIZE_BYTES: usize = 39;
pub const HEADER_SIZE_BITS: usize = HEADER_SIZE_BYTES * 8;
pub const CODING_FACTORS_PER_FRAME: usize =
    CODING_FACTORS_PER_PARTICIPANT_PER_FRAME * MAX_PARTICIPANTS;
pub const CODING_FACTORS_PER_PARTICIPANT_PER_FRAME: usize = 32;
pub const CODING_FACTORS_SIZE_BYTES: usize =
    (CODING_FACTORS_PER_PARTICIPANT_PER_FRAME * GALOIS_FIELD_POWER * MAX_PARTICIPANTS).div_ceil(8);
pub const CODING_FACTOR_OFFSET_SIZE_BITS: usize = 16;
pub const CODING_FACTOR_OFFSET_SIZE_BYTES: usize =
    (CODING_FACTOR_OFFSET_SIZE_BITS * MAX_PARTICIPANTS).div_ceil(8);
pub const CODING_FACTOR_WIDTH_SIZE_BYTES: usize = 16;
pub const NETWORK_CODING_SIZE_BYTES: usize =
    CODING_FACTOR_WIDTH_SIZE_BYTES + CODING_FACTOR_OFFSET_SIZE_BYTES + CODING_FACTORS_SIZE_BYTES;
pub const FRAGMENT_SIZE_BYTES: usize = 515;
pub const FRAGMENTS_PER_PARTICIPANT_PER_EPOCH: usize =
    2usize.pow(CODING_FACTOR_OFFSET_SIZE_BITS as u32) + CODING_FACTORS_PER_PARTICIPANT_PER_FRAME
        - 1;
pub const FRAGMENTS_PER_EPOCH: usize = FRAGMENTS_PER_PARTICIPANT_PER_EPOCH * MAX_PARTICIPANTS;
pub const FRAME_SIZE_BYTES: usize =
    HEADER_SIZE_BYTES + NETWORK_CODING_SIZE_BYTES + FRAGMENT_SIZE_BYTES;

pub const GALOIS_FIELD_POWER: usize = 4;
pub const GALOIS_FIELD: usize = 2usize.pow(GALOIS_FIELD_POWER as u32);

pub const EPOCH_SIZE_BYTES: usize = BYTES_PER_PARTICIPANT * MAX_PARTICIPANTS;
pub const BYTES_PER_PARTICIPANT: usize = FRAGMENT_SIZE_BYTES * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;

pub const QR_CODE_VERSION: Version = Version::Normal(20);
pub const QR_CODE_ECC: EcLevel = EcLevel::L;

pub const AP_LENGTH_INDEX_SIZE_BITS: usize = (FRAGMENT_SIZE_BYTES * CODING_FACTORS_PER_FRAME)
    .next_power_of_two()
    .ilog2() as usize;
pub const AP_LENGTH_INDEX_SIZE_BYTES: usize = AP_LENGTH_INDEX_SIZE_BITS.div_ceil(8);
pub const AP_MAX_SIZE_BYTES: usize =
    FRAGMENT_SIZE_BYTES * CODING_FACTORS_PER_FRAME - AP_LENGTH_INDEX_SIZE_BYTES;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_spec_v4() {
        assert_eq!(MAX_PARTICIPANTS, 16);
        assert_eq!(MAX_PARTICIPANTS_SIZE_BITS, 4);
        assert_eq!(MAX_PARTICIPANTS_SIZE_BYTES, 1);
        assert_eq!(HEADER_SIZE_BYTES, 39);
        assert_eq!(HEADER_SIZE_BITS, 312);
        assert_eq!(CODING_FACTORS_PER_FRAME, 512);
        assert_eq!(CODING_FACTORS_PER_PARTICIPANT_PER_FRAME, 32);
        assert_eq!(CODING_FACTORS_SIZE_BYTES, 256);
        assert_eq!(CODING_FACTOR_OFFSET_SIZE_BITS, 16);
        assert_eq!(CODING_FACTOR_OFFSET_SIZE_BYTES, 32);
        assert_eq!(CODING_FACTOR_WIDTH_SIZE_BYTES, 16);
        assert_eq!(NETWORK_CODING_SIZE_BYTES, 304);
        // assert_eq!(FRAGMENT_SIZE_BYTES, 1024);
        assert_eq!(FRAGMENT_SIZE_BYTES, 515);
        assert_eq!(FRAGMENTS_PER_PARTICIPANT_PER_EPOCH, 65567);
        assert_eq!(FRAGMENTS_PER_EPOCH, 1049072);
        // assert_eq!(FRAME_SIZE_BYTES, 1367);
        assert_eq!(FRAME_SIZE_BYTES, 858);
        assert_eq!(GALOIS_FIELD_POWER, 4);
        assert_eq!(GALOIS_FIELD, 16);
        // assert_eq!(EPOCH_SIZE_BYTES, 1074249728);
        assert_eq!(EPOCH_SIZE_BYTES, 540272080);
        // assert_eq!(BYTES_PER_PARTICIPANT, 67140608);
        assert_eq!(BYTES_PER_PARTICIPANT, 33767005);
        assert_eq!(QR_CODE_VERSION, Version::Normal(20));
        assert_eq!(QR_CODE_ECC, EcLevel::L);
        assert_eq!(AP_LENGTH_INDEX_SIZE_BITS, 19);
        assert_eq!(AP_LENGTH_INDEX_SIZE_BYTES, 3);
        // assert_eq!(AP_MAX_SIZE_BYTES, 524285);
        assert_eq!(AP_MAX_SIZE_BYTES, 263677);
    }
}
