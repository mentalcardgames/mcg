use crate::data_structures::{Fragment, Frame, FrameFactor, FrameHeader};
use crate::network_coding::GaloisField2p4;
use crate::{
    CODING_FACTOR_OFFSET_SIZE_BYTES, CODING_FACTOR_WIDTH_SIZE_BYTES, CODING_FACTORS_PER_FRAME,
    FRAGMENT_SIZE_BYTES, FRAME_SIZE_BYTES, HEADER_SIZE_BYTES, MAX_PARTICIPANTS,
    NETWORK_CODING_SIZE_BYTES,
};
use std::array::from_fn;

impl From<[u8; FRAME_SIZE_BYTES]> for Frame {
    fn from(value: [u8; FRAME_SIZE_BYTES]) -> Self {
        // debug_assert_eq!(FRAME_SIZE_BYTES, 792);
        // debug_assert_eq!(HEADER_SIZE_BYTES, 5);
        // debug_assert_eq!(CODING_FACTOR_OFFSET_SIZE_BYTES, 32);
        // debug_assert_eq!(CODING_FACTORS_SIZE_BYTES, 256);
        // debug_assert_eq!(FRAGMENT_SIZE_BYTES, 483);

        let mut a = 0;
        let mut b = 0;
        let mut header = [0u8; HEADER_SIZE_BYTES];
        let mut coding_factors = [0u8; NETWORK_CODING_SIZE_BYTES];
        let mut fragment = [0u8; FRAGMENT_SIZE_BYTES];
        for slice in [header.as_mut(), coding_factors.as_mut(), fragment.as_mut()] {
            b += slice.len();
            slice.copy_from_slice(&value[a..b]);
            a += slice.len();
        }
        let header = header.into();
        let coding_factors = coding_factors.into();
        let fragment = fragment.into();
        Frame {
            header,
            factors: coding_factors,
            fragment,
        }
    }
}

impl From<[u8; HEADER_SIZE_BYTES]> for FrameHeader {
    fn from(value: [u8; HEADER_SIZE_BYTES]) -> Self {
        // debug_assert_eq!(HEADER_SIZE_BYTES, 5);
        let participant = value[0];
        let is_overflowing = value[1] != 0;
        let epoch = value[2];
        FrameHeader {
            participant,
            is_overflowing,
            epoch,
        }
    }
}

impl From<[u8; NETWORK_CODING_SIZE_BYTES]> for FrameFactor {
    fn from(value: [u8; NETWORK_CODING_SIZE_BYTES]) -> Self {
        let width: [u8; MAX_PARTICIPANTS] = from_fn(|idx| value[idx]);
        let offsets: [u16; MAX_PARTICIPANTS] = from_fn(|idx| {
            u16::from_le_bytes([
                value[MAX_PARTICIPANTS + 2 * idx],
                value[MAX_PARTICIPANTS + 2 * idx + 1],
            ])
        });
        let mut coding_factors = [GaloisField2p4::ZERO; CODING_FACTORS_PER_FRAME];
        let factors: Vec<GaloisField2p4> = value
            [(CODING_FACTOR_OFFSET_SIZE_BYTES + CODING_FACTOR_WIDTH_SIZE_BYTES)..]
            .iter()
            .flat_map(|b| {
                [
                    GaloisField2p4 { inner: *b & 0xF },
                    GaloisField2p4 {
                        inner: (*b & 0xF0) >> 4,
                    },
                ]
            })
            .collect();
        coding_factors[..factors.len()].copy_from_slice(factors.as_slice());
        FrameFactor::new(coding_factors, width, offsets)
            .expect("Seems like the provided values are illegal!")
    }
}
impl From<[u8; FRAGMENT_SIZE_BYTES]> for Fragment {
    fn from(value: [u8; FRAGMENT_SIZE_BYTES]) -> Self {
        let inner = value.into();
        Fragment { inner }
    }
}
