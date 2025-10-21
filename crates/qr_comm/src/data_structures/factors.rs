use crate::network_coding::GaloisField2p4;
use crate::{
    CODING_FACTORS_PER_FRAME, FRAGMENTS_PER_EPOCH, FRAGMENTS_PER_PARTICIPANT_PER_EPOCH,
    MAX_PARTICIPANTS,
};
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct FrameFactor {
    pub widths: [u8; MAX_PARTICIPANTS],
    pub offsets: [u16; MAX_PARTICIPANTS],
    pub factors: [GaloisField2p4; CODING_FACTORS_PER_FRAME],
}

impl Default for FrameFactor {
    fn default() -> Self {
        let widths = [16; MAX_PARTICIPANTS];
        let offsets = [0; MAX_PARTICIPANTS];
        let factors = [GaloisField2p4::ZERO; CODING_FACTORS_PER_FRAME];
        Self {
            widths,
            offsets,
            factors,
        }
    }
}

impl FrameFactor {
    pub fn new(
        factors: [GaloisField2p4; CODING_FACTORS_PER_FRAME],
        widths: [u8; MAX_PARTICIPANTS],
        offsets: [u16; MAX_PARTICIPANTS],
    ) -> Result<Self, &'static str> {
        if widths.iter().fold(0u16, |acc, w| acc + 2 * (*w as u16)) > 512 {
            Err("Illegal widths data. You can't have more than 512 factors!")
        } else {
            Ok(
                FrameFactor {
                    widths,
                    offsets,
                    factors,
                }
            )
        }
    }
    pub fn get_factor_at(&self, idx: usize) -> Option<GaloisField2p4> {
        let participant_idx = idx / FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
        let factor_idx = idx % FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
        let factor_idx_shifted = factor_idx.checked_sub(self.offsets[participant_idx] as usize)?;
        (self.widths[participant_idx] as usize * 2).checked_sub(factor_idx_shifted)?;
        if self.widths[participant_idx] == 0 {
            return None;
        }
        Some(
            self.factors[self.widths[..participant_idx]
                .iter()
                .map(|x| *x as usize * 2)
                .sum::<usize>()
                + factor_idx_shifted],
        )
    }
}

#[derive(PartialEq, Clone, Debug)]
pub struct WideFactor {
    pub inner: Box<[GaloisField2p4; FRAGMENTS_PER_EPOCH]>,
}

impl WideFactor {
    pub fn get_width_and_offsets(&self) -> ([u8; MAX_PARTICIPANTS], [u16; MAX_PARTICIPANTS]) {
        let mut width = [0; MAX_PARTICIPANTS];
        let mut offsets = [0; MAX_PARTICIPANTS];
        for participant in 0..MAX_PARTICIPANTS {
            // Find offset
            for factor in 0..FRAGMENTS_PER_PARTICIPANT_PER_EPOCH {
                if self.inner[participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor]
                    == GaloisField2p4::ZERO
                {
                    continue;
                }
                offsets[participant] = factor as u16;
                break;
            }
            // Find width
            for factor in
                ((offsets[participant] as usize)..FRAGMENTS_PER_PARTICIPANT_PER_EPOCH).rev()
            {
                if self.inner[participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor]
                    == GaloisField2p4::ZERO
                {
                    continue;
                }
                width[participant] = (factor as u16 - offsets[participant]).div_ceil(2) as u8;
                break;
            }
        }
        (width, offsets)
    }
    pub fn utilized_fragments(&self) -> Box<[bool; FRAGMENTS_PER_EPOCH]> {
        let mut utilization: Box<[bool; FRAGMENTS_PER_EPOCH]> = vec![false; FRAGMENTS_PER_EPOCH]
            .try_into()
            .expect("Error allocating memory!");
        let util: Vec<bool> = self
            .inner
            .iter()
            .map(|f| *f != GaloisField2p4::ZERO)
            .collect();
        utilization.copy_from_slice(util.as_slice());
        utilization
    }
    pub fn is_plain(&self) -> bool {
        self.inner
            .iter()
            .filter(|&f| *f != GaloisField2p4::ZERO)
            .count()
            == 1
    }
}

impl Default for WideFactor {
    fn default() -> Self {
        let inner = vec![GaloisField2p4::ZERO; FRAGMENTS_PER_EPOCH]
            .try_into()
            .expect("Error allocating memory!");
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
            value.widths.iter().zip(value.offsets.iter()).enumerate()
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

#[allow(dead_code)]
#[derive(PartialEq, Clone, Debug)]
pub struct CompactFactor {
    pub inner: Vec<(usize, GaloisField2p4)>,
}

impl Default for CompactFactor {
    fn default() -> Self {
        let inner = Vec::new();
        CompactFactor { inner }
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structures::FrameFactor;
    use crate::network_coding::GaloisField2p4;
    use crate::{
        CODING_FACTORS_PER_PARTICIPANT_PER_FRAME, FRAGMENTS_PER_PARTICIPANT_PER_EPOCH,
        MAX_PARTICIPANTS,
    };

    #[test]
    fn get_factor_at_test_0() {
        let mut factors = FrameFactor::default();
        for participant in 0..MAX_PARTICIPANTS {
            for factor in 0..10 {
                factors.factors[participant * CODING_FACTORS_PER_PARTICIPANT_PER_FRAME + factor] =
                    GaloisField2p4::from((factor + participant) as u8 & 0xF);
            }
        }
        for participant in 0..MAX_PARTICIPANTS {
            for factor in 0..10 {
                let f = factors
                    .get_factor_at(participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor);
                let expected = GaloisField2p4::from((factor + participant) as u8 & 0xF);
                assert!(
                    f.is_some(),
                    "Missing for participant {} at factor {}",
                    participant,
                    factor
                );
                assert_eq!(f.unwrap(), expected);
            }
        }
        factors.widths[4] = 0;
        factors.widths[MAX_PARTICIPANTS-1] *= 2;
        for factor in 0..10 {
            let f = factors.get_factor_at(4 * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor);
            assert!(f.is_none());
        }
        for participant in 5..(MAX_PARTICIPANTS-1) {
            for factor in 0..10 {
                let f =
                    factors.get_factor_at(participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor);
                let expected = GaloisField2p4::from((factor + participant - 1) as u8  & 0xF);
                assert!(
                    f.is_some(),
                    "Missing for participant {} at factor {}",
                    participant,
                    factor
                );
                assert_eq!(f.unwrap(), expected);
            }
        }
        for factor in 0..10usize {
            let f = factors.get_factor_at((MAX_PARTICIPANTS-1) * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor);
            let expected = GaloisField2p4::from((factor + MAX_PARTICIPANTS-2) as u8 & 0xF);
            assert!(f.is_some());
            assert_eq!(f.unwrap(), expected);
        }
        for factor in 0..10usize {
            let f = factors.get_factor_at((MAX_PARTICIPANTS-1) * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + factor + CODING_FACTORS_PER_PARTICIPANT_PER_FRAME);
            let expected = GaloisField2p4::from((factor + MAX_PARTICIPANTS-1) as u8 & 0xF);
            assert!(f.is_some());
            assert_eq!(f.unwrap(), expected);
        }
    }
}
