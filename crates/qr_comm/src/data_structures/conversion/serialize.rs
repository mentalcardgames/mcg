use crate::data_structures::{CodingFactor, Fragment, Frame, FrameHeader};
use crate::{CODING_FACTORS_SIZE, FRAGMENT_SIZE_BYTES, FRAME_SIZE_BYTES, HEADER_SIZE_BYTES};

impl From<Frame> for [u8; FRAME_SIZE_BYTES] {
    fn from(val: Frame) -> Self {
        debug_assert_eq!(CODING_FACTORS_SIZE, 688);
        debug_assert_eq!(FRAGMENT_SIZE_BYTES, 676);
        debug_assert_eq!(HEADER_SIZE_BYTES, 3);
        debug_assert_eq!(
            CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES + HEADER_SIZE_BYTES,
            FRAME_SIZE_BYTES
        );
        let mut result: [u8; FRAME_SIZE_BYTES] = [0u8; FRAME_SIZE_BYTES];
        let Frame {
            coding_factor,
            fragment,
            header,
        } = val;
        let coding_factor: [u8; CODING_FACTORS_SIZE] = coding_factor.into();
        let a = 0;
        let b = CODING_FACTORS_SIZE;
        result[a..b].copy_from_slice(&coding_factor);
        let fragment: [u8; FRAGMENT_SIZE_BYTES] = fragment.into();
        let a = CODING_FACTORS_SIZE;
        let b = CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES;
        result[a..b].copy_from_slice(&fragment);
        let header: [u8; HEADER_SIZE_BYTES] = header.into();
        let a = CODING_FACTORS_SIZE + FRAGMENT_SIZE_BYTES;
        let b = FRAME_SIZE_BYTES;
        result[a..b].copy_from_slice(&header);
        result
    }
}

impl From<FrameHeader> for [u8; HEADER_SIZE_BYTES] {
    fn from(val: FrameHeader) -> Self {
        debug_assert_eq!(HEADER_SIZE_BYTES, 3);
        let mut result = [0u8; HEADER_SIZE_BYTES];
        let FrameHeader {
            participant: sender_id,
            is_overflowing,
            epoch,
        } = val;
        result[0] = sender_id;
        result[1] = is_overflowing as u8;
        result[2] = epoch;
        result
    }
}

impl From<CodingFactor> for [u8; CODING_FACTORS_SIZE] {
    fn from(val: CodingFactor) -> Self {
        val.inner
    }
}

impl From<Fragment> for [u8; FRAGMENT_SIZE_BYTES] {
    fn from(val: Fragment) -> Self {
        val.inner
    }
}
