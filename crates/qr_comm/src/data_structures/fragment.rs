use crate::FRAGMENT_SIZE_BYTES;
use crate::network_coding::GaloisField2p4;
use std::ops::{AddAssign, Deref, DerefMut, Mul, MulAssign};

#[derive(Clone, PartialEq, Debug)]
pub struct Fragment {
    pub inner: Box<[u8; FRAGMENT_SIZE_BYTES]>,
}

impl Default for Fragment {
    fn default() -> Self {
        let inner = vec![0u8; FRAGMENT_SIZE_BYTES]
            .try_into()
            .expect("Error allocating memory!");
        Fragment { inner }
    }
}
impl Deref for Fragment {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl DerefMut for Fragment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}
impl MulAssign<GaloisField2p4> for Fragment {
    fn mul_assign(&mut self, rhs: GaloisField2p4) {
        self.inner.iter_mut().for_each(|f| {
            let upper = GaloisField2p4 {
                inner: (*f & 0xF0) >> 4,
            } * rhs;
            let lower = GaloisField2p4 { inner: *f & 0xF } * rhs;
            *f = (upper.inner << 4) | lower.inner
        });
    }
}
impl Mul<GaloisField2p4> for Fragment {
    type Output = Fragment;

    fn mul(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl AddAssign<Fragment> for Fragment {
    fn add_assign(&mut self, rhs: Fragment) {
        self.inner
            .iter_mut()
            .zip(rhs.inner.iter())
            .for_each(|(lhs, rhs)| {
                *lhs ^= rhs;
            });
    }
}

impl Fragment {
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.iter().all(|&x| x == 0)
    }
}
