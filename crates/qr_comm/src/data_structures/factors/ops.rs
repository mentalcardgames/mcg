use crate::data_structures::{Factor, SparseFactor, WideFactor};
use crate::network_coding::GaloisField2p4;
use std::mem;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

// std::ops for WideFactor
impl SubAssign<WideFactor> for WideFactor {
    fn sub_assign(&mut self, rhs: Self) {
        self.inner
            .iter_mut()
            .zip(rhs.inner.iter())
            .for_each(|(lhs, rhs)| {
                *lhs -= *rhs;
            });
    }
}
impl Sub<WideFactor> for WideFactor {
    type Output = WideFactor;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs -= rhs;
        lhs
    }
}
impl AddAssign<WideFactor> for WideFactor {
    fn add_assign(&mut self, rhs: Self) {
        self.inner
            .iter_mut()
            .zip(rhs.inner.iter())
            .for_each(|(lhs, rhs)| {
                *lhs += *rhs;
            });
    }
}
impl Add<WideFactor> for WideFactor {
    type Output = WideFactor;

    fn add(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs += rhs;
        lhs
    }
}
impl MulAssign<GaloisField2p4> for WideFactor {
    fn mul_assign(&mut self, rhs: GaloisField2p4) {
        self.inner.iter_mut().for_each(|f| *f *= rhs);
    }
}
impl MulAssign<u8> for WideFactor {
    fn mul_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self *= rhs;
    }
}
impl Mul<GaloisField2p4> for WideFactor {
    type Output = WideFactor;

    fn mul(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl Mul<u8> for WideFactor {
    type Output = WideFactor;

    fn mul(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl DivAssign<GaloisField2p4> for WideFactor {
    fn div_assign(&mut self, rhs: GaloisField2p4) {
        self.inner.iter_mut().for_each(|f| *f /= rhs);
    }
}
impl DivAssign<u8> for WideFactor {
    fn div_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self /= rhs;
    }
}
impl Div<GaloisField2p4> for WideFactor {
    type Output = WideFactor;

    fn div(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}
impl Div<u8> for WideFactor {
    type Output = WideFactor;

    fn div(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}

// std::ops for SparseFactor
impl SubAssign<SparseFactor> for SparseFactor {
    fn sub_assign(&mut self, rhs: Self) {
        let mut idx_lhs = 0;
        let mut idx_rhs = 0;

        let len_rhs = rhs.inner.len();
        while idx_lhs < self.inner.len() && idx_rhs < len_rhs {
            let (factor_idx_lhs, factor_lhs) = self.inner[idx_lhs];
            let (factor_idx_rhs, factor_rhs) = rhs.inner[idx_rhs];

            if factor_idx_lhs == factor_idx_rhs {
                if factor_lhs == factor_rhs {
                    // Remove union element
                    self.inner.remove(idx_lhs);
                } else {
                    self.inner[idx_lhs].1 -= factor_rhs;
                    idx_lhs += 1;
                }
                idx_rhs += 1;
            } else if factor_idx_rhs < factor_idx_lhs {
                self.inner.insert(idx_lhs, (factor_idx_rhs, factor_rhs));
                idx_rhs += 1;
                idx_lhs += 1;
            } else {
                idx_lhs += 1;
            }
        }

        // Add remaining elements
        self.inner.extend(&rhs.inner[idx_rhs..]);
    }
}
impl Sub<SparseFactor> for SparseFactor {
    type Output = SparseFactor;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs -= rhs;
        lhs
    }
}

impl AddAssign<SparseFactor> for SparseFactor {
    fn add_assign(&mut self, rhs: Self) {
        self.sub_assign(rhs);
    }
}
impl Add<SparseFactor> for SparseFactor {
    type Output = SparseFactor;

    fn add(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs += rhs;
        lhs
    }
}
impl MulAssign<GaloisField2p4> for SparseFactor {
    fn mul_assign(&mut self, rhs: GaloisField2p4) {
        if rhs == GaloisField2p4::ZERO {
            mem::take(&mut self.inner);
        } else {
            self.inner.iter_mut().for_each(|(_, f)| *f *= rhs);
        }
    }
}
impl MulAssign<u8> for SparseFactor {
    fn mul_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self *= rhs;
    }
}
impl Mul<GaloisField2p4> for SparseFactor {
    type Output = SparseFactor;

    fn mul(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl Mul<u8> for SparseFactor {
    type Output = SparseFactor;

    fn mul(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl DivAssign<GaloisField2p4> for SparseFactor {
    fn div_assign(&mut self, rhs: GaloisField2p4) {
        if rhs == GaloisField2p4::ZERO {
            mem::take(&mut self.inner);
        } else {
            self.inner.iter_mut().for_each(|(_, f)| *f /= rhs);
        }
    }
}
impl DivAssign<u8> for SparseFactor {
    fn div_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self /= rhs;
    }
}
impl Div<GaloisField2p4> for SparseFactor {
    type Output = SparseFactor;

    fn div(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}
impl Div<u8> for SparseFactor {
    type Output = SparseFactor;

    fn div(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}

// std::ops for Factor
impl SubAssign<Factor> for Factor {
    fn sub_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Factor::Wide(lhs), Factor::Wide(rhs)) => lhs.sub_assign(rhs),
            (Factor::Sparse(lhs), Factor::Sparse(rhs)) => lhs.sub_assign(rhs),
            (Factor::Wide(lhs), Factor::Sparse(rhs)) => {
                let mut lhs: SparseFactor = lhs.to_owned().into();
                lhs.sub_assign(rhs);
            }
            (Factor::Sparse(lhs), Factor::Wide(rhs)) => {
                let rhs: SparseFactor = rhs.into();
                lhs.sub_assign(rhs);
            }
        }
    }
}
impl Sub<Factor> for Factor {
    type Output = Factor;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs -= rhs;
        lhs
    }
}
impl AddAssign<Factor> for Factor {
    fn add_assign(&mut self, rhs: Self) {
        match (self, rhs) {
            (Factor::Wide(lhs), Factor::Wide(rhs)) => lhs.add_assign(rhs),
            (Factor::Sparse(lhs), Factor::Sparse(rhs)) => lhs.add_assign(rhs),
            (Factor::Wide(lhs), Factor::Sparse(rhs)) => {
                let mut lhs: SparseFactor = lhs.to_owned().into();
                lhs.add_assign(rhs);
                // *self = Factor::Sparse(lhs);
            }
            (Factor::Sparse(lhs), Factor::Wide(rhs)) => {
                let rhs: SparseFactor = rhs.to_owned().into();
                lhs.add_assign(rhs);
            }
        }
    }
}
impl Add<Factor> for Factor {
    type Output = Factor;

    fn add(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs += rhs;
        lhs
    }
}
impl MulAssign<GaloisField2p4> for Factor {
    fn mul_assign(&mut self, rhs: GaloisField2p4) {
        match self {
            Factor::Sparse(lhs) => lhs.mul_assign(rhs),
            Factor::Wide(lhs) => lhs.mul_assign(rhs),
        }
    }
}
impl MulAssign<u8> for Factor {
    fn mul_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self *= rhs;
    }
}
impl Mul<GaloisField2p4> for Factor {
    type Output = Factor;

    fn mul(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl Mul<u8> for Factor {
    type Output = Factor;

    fn mul(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl DivAssign<GaloisField2p4> for Factor {
    fn div_assign(&mut self, rhs: GaloisField2p4) {
        match self {
            Factor::Sparse(lhs) => lhs.div_assign(rhs),
            Factor::Wide(lhs) => lhs.div_assign(rhs),
        }
    }
}
impl DivAssign<u8> for Factor {
    fn div_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self /= rhs;
    }
}
impl Div<GaloisField2p4> for Factor {
    type Output = Factor;

    fn div(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}
impl Div<u8> for Factor {
    type Output = Factor;

    fn div(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn add_wide() {

    }
    #[test]
    fn add_sparse() {}
    #[test]
    fn add_factor() {}
}
