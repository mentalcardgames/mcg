use crate::data_structures::{CodingFactor, Fragment, Frame, FrameHeader};
use crate::{FRAGMENT_SIZE_BYTES, FRAME_SIZE_BYTES, HEADER_SIZE_BYTES, CODING_FACTORS_SIZE};

impl From<[u8; FRAME_SIZE_BYTES]> for Frame {
    fn from(value: [u8; FRAME_SIZE_BYTES]) -> Self {
        debug_assert_eq!(CODING_FACTORS_SIZE, 688);
        debug_assert_eq!(FRAGMENT_SIZE_BYTES, 676);
        debug_assert_eq!(HEADER_SIZE_BYTES, 3);
        debug_assert_eq!(
            CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES + HEADER_SIZE_BYTES,
            FRAME_SIZE_BYTES
        );
        let mut coding_factor = [0u8; CODING_FACTORS_SIZE];
        let a = 0;
        let b = CODING_FACTORS_SIZE;
        coding_factor.copy_from_slice(&value[a..b]);
        let coding_factor = coding_factor.into();
        let mut fragment = [0u8; FRAGMENT_SIZE_BYTES];
        let a = CODING_FACTORS_SIZE;
        let b = CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES;
        fragment.copy_from_slice(&value[a..b]);
        let fragment = fragment.into();
        let mut header = [0u8; HEADER_SIZE_BYTES];
        let a = CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES;
        let b = FRAME_SIZE_BYTES;
        header.copy_from_slice(&value[a..b]);
        let header = header.into();
        Frame { coding_factor, fragment, header }
    }
}

impl From<[u8; HEADER_SIZE_BYTES]> for FrameHeader {
    fn from(value: [u8; HEADER_SIZE_BYTES]) -> Self {
        debug_assert_eq!(HEADER_SIZE_BYTES, 3);
        let sender_id = value[0];
        let is_overflowing = value[1] != 0;
        let epoch = value[2];
        FrameHeader {
            participant: sender_id,
            is_overflowing,
            epoch,
        }
    }
}

impl From<[u8; CODING_FACTORS_SIZE]> for CodingFactor {
    fn from(value: [u8; CODING_FACTORS_SIZE]) -> Self {
        CodingFactor::new(value)
    }
}
impl From<[u8; FRAGMENT_SIZE_BYTES]> for Fragment {
    fn from(value: [u8; FRAGMENT_SIZE_BYTES]) -> Self {
        Fragment { inner: value }
    }
}
