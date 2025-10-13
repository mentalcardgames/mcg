use qrcode::{EcLevel, Version};

pub mod network_coding;
pub mod data_structures;

pub const FRAGMENT_SIZE_BYTES: usize = 676;
pub const FRAGMENTS_PER_PARTICIPANT: usize = EPOCH_SIZE_FRAGMENTS.div_euclid(MAX_PARTICIPANTS);
pub const BYTES_PER_PARTICIPANT: usize = EPOCH_SIZE_BYTES.div_euclid(MAX_PARTICIPANTS);
pub const MAX_PARTICIPANTS: usize = 16;
pub const MAX_PARTICIPANTS_SIZE_BITS: usize = MAX_PARTICIPANTS.next_power_of_two().ilog2() as usize;
pub const MAX_PARTICIPANTS_SIZE_BYTES: usize = MAX_PARTICIPANTS_SIZE_BITS.div_ceil(8);
pub const CODING_FACTORS_SIZE: usize = EPOCH_SIZE_FRAGMENTS;
pub const HEADER_SIZE_BITS: usize = HEADER_SIZE_BYTES * 8;
pub const HEADER_SIZE_BYTES: usize = 3;
pub const FRAME_SIZE_BYTES: usize = CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES + HEADER_SIZE_BYTES;
pub const QR_CODE_VERSION: Version = Version::Normal(26);
pub const QR_CODE_ECC: EcLevel = EcLevel::L;
pub const GALOIS_FIELD: usize = 256;
pub const EPOCH_SIZE_BYTES: usize = FRAGMENT_SIZE_BYTES * EPOCH_SIZE_FRAGMENTS;
pub const EPOCH_SIZE_FRAGMENTS: usize = 688;

#[cfg(test)]
mod test_constants {
    use super::*;
    #[test]
    fn test_spec_v0() {
        assert_eq!(FRAGMENT_SIZE_BYTES, 676);
        assert_eq!(FRAGMENTS_PER_PARTICIPANT, 43);
        assert_eq!(BYTES_PER_PARTICIPANT, 29068);
        assert_eq!(MAX_PARTICIPANTS, 16);
        assert_eq!(MAX_PARTICIPANTS_SIZE_BITS, 4);
        assert_eq!(MAX_PARTICIPANTS_SIZE_BYTES, 1);
        assert_eq!(CODING_FACTORS_SIZE, 688);
        assert_eq!(HEADER_SIZE_BITS, 24);
        assert_eq!(HEADER_SIZE_BYTES, 3);
        assert_eq!(FRAME_SIZE_BYTES, 1367);
        assert_eq!(QR_CODE_VERSION, Version::Normal(26));
        assert_eq!(QR_CODE_ECC, EcLevel::L);
        assert_eq!(GALOIS_FIELD, 256);
        assert_eq!(EPOCH_SIZE_BYTES, 465088);
        assert_eq!(EPOCH_SIZE_FRAGMENTS, 688);
    }
}
