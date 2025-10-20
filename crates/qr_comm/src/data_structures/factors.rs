use crate::network_coding::GaloisField2p4;
use crate::{
    CODING_FACTORS_PER_FRAME, CODING_FACTORS_SIZE_BYTES, FRAGMENTS_PER_EPOCH,
    FRAGMENTS_PER_PARTICIPANT_PER_EPOCH, CODING_FACTORS_PER_PARTICIPANT_PER_FRAME, MAX_PARTICIPANTS,
};
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameFactor {
    pub width: [u8; MAX_PARTICIPANTS],
    pub offsets: [u16; MAX_PARTICIPANTS],
    pub factors: [GaloisField2p4; CODING_FACTORS_PER_FRAME],
}

impl Default for FrameFactor {
    fn default() -> Self {
        let width = [16; MAX_PARTICIPANTS];
        let offsets = [0; MAX_PARTICIPANTS];
        let factors = [GaloisField2p4::ZERO; CODING_FACTORS_PER_FRAME];
        Self {
            width,
            offsets,
            factors,
        }
    }
}
impl FrameFactor {
    pub fn new(factors: [GaloisField2p4; CODING_FACTORS_PER_FRAME], width: [u8; MAX_PARTICIPANTS], offsets: [u16; MAX_PARTICIPANTS]) -> Self {
        if width.iter().fold(0u16, |acc, w| { acc + 2*(*w as u16)}) != 512 {
            panic!("Width data is illegal. You need to use all 512 factors!");
        }
        let _data: Vec<GaloisField2p4> = [0; CODING_FACTORS_SIZE_BYTES]
            .iter()
            .flat_map(|b| {
                let upper = *b | 0xF0 >> 4;
                let lower = *b & 0xF;
                [GaloisField2p4::new(lower), GaloisField2p4::new(upper)]
            })
            .collect();
        FrameFactor { width, offsets, factors }
    }
    pub fn get_factor_at(&self, idx: usize) -> GaloisField2p4 {
        let participant_idx = idx / FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
        let factor_idx = idx % FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
        if let Some(idx) = factor_idx.checked_sub(self.offsets[participant_idx] as usize)
            && let Some(idx) = idx.checked_sub(CODING_FACTORS_PER_PARTICIPANT_PER_FRAME)
        {
            self.factors[CODING_FACTORS_PER_PARTICIPANT_PER_FRAME * participant_idx + idx]
        } else {
            GaloisField2p4::ZERO
        }
    }
}

#[derive(PartialEq, Copy, Clone)]
pub struct WideFactor {
    pub inner: [GaloisField2p4; FRAGMENTS_PER_EPOCH],
}

impl WideFactor {
    pub fn get_width_and_offsets(&self) -> ([u8; MAX_PARTICIPANTS], [u16; MAX_PARTICIPANTS]) {
        let mut width = [0; MAX_PARTICIPANTS];
        let mut offsets = [0; MAX_PARTICIPANTS];
        for participant in 0..MAX_PARTICIPANTS {
            // Find offset
            for factor in 0..FRAGMENTS_PER_PARTICIPANT_PER_EPOCH {
                if self.inner[participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor] == GaloisField2p4::ZERO {
                    continue;
                }
                offsets[participant] = factor as u16;
                break;
            }
            // Find width
            for factor in ((offsets[participant] as usize)..FRAGMENTS_PER_PARTICIPANT_PER_EPOCH).rev() {
                if self.inner[participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor] == GaloisField2p4::ZERO {
                    continue;
                }
                width[participant] = (factor as u16 - offsets[participant]).div_ceil(2) as u8;
                break;
            }
        }
        (width, offsets)
    }
}

impl Default for WideFactor {
    fn default() -> Self {
        let inner = [GaloisField2p4::ZERO; FRAGMENTS_PER_EPOCH];
        WideFactor { inner }
    }
}
impl Deref for WideFactor {
    type Target = [GaloisField2p4];

    fn deref(&self) -> &Self::Target {
        self.inner.as_ref()
    }
}
impl DerefMut for WideFactor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.as_mut()
    }
}
impl From<FrameFactor> for WideFactor {
    fn from(value: FrameFactor) -> Self {
        let mut wide = WideFactor::default();
        let mut start: usize = 0;
        let mut stop: usize = 0;
        for (participant, (width, offset)) in
            value.width.iter().zip(value.offsets.iter()).enumerate()
        {
            stop += 2 * (*width as usize);
            let a = participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + (*offset as usize);
            let b = a + 2 * (*width as usize);
            wide.inner[a..b].copy_from_slice(value.factors[start..stop].as_ref());
            start += 2 * (*width as usize);
        }
        wide
    }
}
