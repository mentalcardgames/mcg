use crate::data_structures::factors::WideFactor;
use crate::data_structures::Fragment;
use crate::network_coding::GaloisField2p4;
use crate::FRAGMENTS_PER_EPOCH;

#[derive(Copy, Clone, Default)]
pub struct Equation {
    pub factors: WideFactor,
    pub fragment: Fragment,
}

impl Equation {
    pub fn new(factors: WideFactor, fragment: Fragment) -> Self {
        Equation { factors, fragment }
    }

    pub fn utilized_fragments(&self) -> [bool; FRAGMENTS_PER_EPOCH] {
        let mut utilization = [false; FRAGMENTS_PER_EPOCH];
        let util: Vec<bool> = self
            .factors
            .iter()
            .map(|f| *f != GaloisField2p4::ZERO)
            .collect();
        utilization.copy_from_slice(util.as_slice());
        utilization
    }

    pub fn is_plain(&self) -> bool {
        self.factors
            .iter()
            .filter(|&f| *f != GaloisField2p4::ZERO)
            .count()
            == 1
    }
}

/// Mathematical operations
impl Equation {
    pub fn div_assign(&mut self, denominator: u8) {
        self.factors.inner = self.factors.inner.map(|f| f / denominator);
        self.fragment.inner = self.fragment.inner.map(|f| {
            let upper = (f & 0xF0) >> 4;
            let lower = f & 0xF;
            let upper = GaloisField2p4::new(upper) / denominator;
            let lower = GaloisField2p4::new(lower) / denominator;
            let upper = upper.inner << 4;
            let lower = lower.inner;
            upper | lower
        });
    }
    pub fn mul_assign(&mut self, factor: u8) {
        self.factors.inner = self.factors.inner.map(|f| f * factor);
        self.fragment.inner = self.fragment.inner.map(|f| {
            let upper = (f & 0xF0) >> 4;
            let lower = f & 0xF;
            let upper = GaloisField2p4::new(upper) * factor;
            let lower = GaloisField2p4::new(lower) * factor;
            let upper = upper.inner << 4;
            let lower = lower.inner;
            upper | lower
        });
    }
    pub fn sub_scaled_assign(&mut self, scale: u8, rhs: &Self) {
        let mut rhs = *rhs;
        rhs.mul_assign(scale);
        self.factors
            .inner
            .iter_mut()
            .enumerate()
            .for_each(|(i, f)| {
                *f -= rhs.factors[i];
            });
        self.fragment
            .inner
            .iter_mut()
            .enumerate()
            .for_each(|(i, f)| {
                *f ^= rhs.fragment[i];
            });
    }
    pub fn add_scaled_assign(&mut self, scale: u8, rhs: &Self) {
        let mut rhs = *rhs;
        rhs.mul_assign(scale);
        self.factors
            .inner
            .iter_mut()
            .enumerate()
            .for_each(|(i, f)| {
                *f += rhs.factors[i];
            });
        self.fragment
            .inner
            .iter_mut()
            .enumerate()
            .for_each(|(i, f)| {
                *f ^= rhs.fragment[i];
            });
    }
}
