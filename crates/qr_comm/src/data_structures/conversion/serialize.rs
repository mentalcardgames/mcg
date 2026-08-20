use crate::data_structures::{Fragment, Frame, FrameFactor, FrameHeader};
use crate::{
    CODING_FACTOR_OFFSET_SIZE_BYTES, CODING_FACTOR_WIDTH_SIZE_BYTES, CODING_FACTORS_SIZE_BYTES,
    FRAGMENT_SIZE_BYTES, FRAME_SIZE_BYTES, HEADER_SIZE_BYTES, MAX_PARTICIPANTS,
    NETWORK_CODING_SIZE_BYTES,
};
use std::array::from_fn;

impl From<Frame> for [u8; FRAME_SIZE_BYTES] {
    fn from(val: Frame) -> Self {
        // debug_assert_eq!(HEADER_SIZE_BYTES, 5);
        // debug_assert_eq!(CODING_FACTOR_OFFSET_SIZE_BYTES, 32);
        // debug_assert_eq!(CODING_FACTORS_SIZE_BYTES, 256);
        // debug_assert_eq!(FRAGMENT_SIZE_BYTES, 483);

        let mut result: [u8; FRAME_SIZE_BYTES] = [0u8; FRAME_SIZE_BYTES];
        let Frame {
            factors: coding_factors,
            fragment,
            header,
        } = val;
        let header: [u8; HEADER_SIZE_BYTES] = header.into();
        let coding_factors: [u8; NETWORK_CODING_SIZE_BYTES] = coding_factors.into();
        let fragment: [u8; FRAGMENT_SIZE_BYTES] = fragment.into();
        let mut a = 0;
        let mut b = 0;
        for slice in [
            header.as_slice(),
            coding_factors.as_slice(),
            fragment.as_slice(),
        ] {
            b += slice.len();
            result[a..b].copy_from_slice(slice);
            a += slice.len();
        }
        result
    }
}

impl From<FrameHeader> for [u8; HEADER_SIZE_BYTES] {
    fn from(val: FrameHeader) -> Self {
        // debug_assert_eq!(HEADER_SIZE_BYTES, 5);
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

impl From<FrameFactor> for [u8; NETWORK_CODING_SIZE_BYTES] {
    fn from(val: FrameFactor) -> Self {
        let FrameFactor {
            widths,
            offsets,
            factors,
        } = val;
        let mut result = [0; NETWORK_CODING_SIZE_BYTES];
        for idx in 0..MAX_PARTICIPANTS {
            result[idx] = widths[idx];
            let [upper, lower] = offsets[idx].to_le_bytes();
            result[MAX_PARTICIPANTS + 2 * idx] = lower;
            result[MAX_PARTICIPANTS + 2 * idx + 1] = upper;
        }
        let factors: [u8; CODING_FACTORS_SIZE_BYTES] = from_fn(|idx| {
            let lower = factors[2 * idx].inner;
            let upper = factors[2 * idx + 1].inner << 4;
            lower | upper
        });
        result[CODING_FACTOR_OFFSET_SIZE_BYTES + CODING_FACTOR_WIDTH_SIZE_BYTES..]
            .copy_from_slice(&factors);
        result
    }
}

impl From<Fragment> for [u8; FRAGMENT_SIZE_BYTES] {
    fn from(val: Fragment) -> Self {
        *val.inner
    }
}
