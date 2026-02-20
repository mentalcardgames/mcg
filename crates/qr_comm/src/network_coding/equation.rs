use crate::data_structures::{Factor, Fragment, SparseFactor};
use crate::network_coding::GaloisField2p4;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Clone)]
pub struct Equation {
    pub factors: Factor,
    pub fragment: Fragment,
}

impl Equation {
    pub fn new(factors: impl Into<Factor>, fragment: Fragment) -> Self {
        let factors = factors.into();
        Equation { factors, fragment }
    }
    pub fn plain_at_index(index: usize, fragment: Fragment) -> Self {
        let mut sparse = SparseFactor::default();
        sparse.inner.push((index, GaloisField2p4::ONE));
        let factors = Factor::Sparse(sparse);
        Equation { factors, fragment }
    }
}

impl SubAssign<Equation> for Equation {
    fn sub_assign(&mut self, rhs: Self) {
        self.factors -= rhs.factors;
        self.fragment -= rhs.fragment;
    }
}
impl Sub<Equation> for Equation {
    type Output = Equation;

    fn sub(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs -= rhs;
        lhs
    }
}
impl AddAssign<Equation> for Equation {
    fn add_assign(&mut self, rhs: Self) {
        self.factors += rhs.factors;
        self.fragment += rhs.fragment;
    }
}
impl Add<Equation> for Equation {
    type Output = Equation;

    fn add(self, rhs: Self) -> Self::Output {
        let mut lhs = self;
        lhs += rhs;
        lhs
    }
}
impl MulAssign<GaloisField2p4> for Equation {
    fn mul_assign(&mut self, rhs: GaloisField2p4) {
        self.factors *= rhs;
        self.fragment *= rhs;
    }
}
impl MulAssign<u8> for Equation {
    fn mul_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self *= rhs;
    }
}
impl Mul<GaloisField2p4> for Equation {
    type Output = Equation;

    fn mul(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl Mul<u8> for Equation {
    type Output = Equation;

    fn mul(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs *= rhs;
        lhs
    }
}
impl DivAssign<GaloisField2p4> for Equation {
    fn div_assign(&mut self, rhs: GaloisField2p4) {
        self.factors /= rhs;
        self.fragment /= rhs;
    }
}
impl DivAssign<u8> for Equation {
    fn div_assign(&mut self, rhs: u8) {
        let rhs = GaloisField2p4::from(rhs);
        *self /= rhs;
    }
}
impl Div<GaloisField2p4> for Equation {
    type Output = Equation;

    fn div(self, rhs: GaloisField2p4) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}
impl Div<u8> for Equation {
    type Output = Equation;

    fn div(self, rhs: u8) -> Self::Output {
        let mut lhs = self;
        lhs /= rhs;
        lhs
    }
}
