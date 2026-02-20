use rand::Rng;
use rand::distr::{Distribution, StandardUniform};
use rand::seq::IndexedRandom;
use std::array::from_fn;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

const MUL_TABLE_2D: [[u8; 16]; 16] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
    [0, 2, 4, 6, 8, 10, 12, 14, 3, 1, 7, 5, 11, 9, 15, 13],
    [0, 3, 6, 5, 12, 15, 10, 9, 11, 8, 13, 14, 7, 4, 1, 2],
    [0, 4, 8, 12, 3, 7, 11, 15, 6, 2, 14, 10, 5, 1, 13, 9],
    [0, 5, 10, 15, 7, 2, 13, 8, 14, 11, 4, 1, 9, 12, 3, 6],
    [0, 6, 12, 10, 11, 13, 7, 1, 5, 3, 9, 15, 14, 8, 2, 4],
    [0, 7, 14, 9, 15, 8, 1, 6, 13, 10, 3, 4, 2, 5, 12, 11],
    [0, 8, 3, 11, 6, 14, 5, 13, 12, 4, 15, 7, 10, 2, 9, 1],
    [0, 9, 1, 8, 2, 11, 3, 10, 4, 13, 5, 12, 6, 15, 7, 14],
    [0, 10, 7, 13, 14, 4, 9, 3, 15, 5, 8, 2, 1, 11, 6, 12],
    [0, 11, 5, 14, 10, 1, 15, 4, 7, 12, 2, 9, 13, 6, 8, 3],
    [0, 12, 11, 7, 5, 9, 14, 2, 10, 6, 1, 13, 15, 3, 4, 8],
    [0, 13, 9, 4, 1, 12, 8, 5, 2, 15, 11, 6, 3, 14, 10, 7],
    [0, 14, 15, 1, 13, 3, 2, 12, 9, 7, 6, 8, 4, 10, 11, 5],
    [0, 15, 13, 2, 9, 6, 4, 11, 1, 14, 12, 3, 8, 7, 5, 10],
];
const DIV_TABLE_2D: [[u8; 16]; 16] = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 9, 14, 13, 11, 7, 6, 15, 2, 12, 5, 10, 4, 3, 8],
    [0, 2, 1, 15, 9, 5, 14, 12, 13, 4, 11, 10, 7, 8, 6, 3],
    [0, 3, 8, 1, 4, 14, 9, 10, 2, 6, 7, 15, 13, 12, 5, 11],
    [0, 4, 2, 13, 1, 10, 15, 11, 9, 8, 5, 7, 14, 3, 12, 6],
    [0, 5, 11, 3, 12, 1, 8, 13, 6, 10, 9, 2, 4, 7, 15, 14],
    [0, 6, 3, 2, 8, 15, 1, 7, 4, 12, 14, 13, 9, 11, 10, 5],
    [0, 7, 10, 12, 5, 4, 6, 1, 11, 14, 2, 8, 3, 15, 9, 13],
    [0, 8, 4, 9, 2, 7, 13, 5, 1, 3, 10, 14, 15, 6, 11, 12],
    [0, 9, 13, 7, 15, 12, 10, 3, 14, 1, 6, 11, 5, 2, 8, 4],
    [0, 10, 5, 6, 11, 2, 3, 9, 12, 7, 1, 4, 8, 14, 13, 15],
    [0, 11, 12, 8, 6, 9, 4, 15, 3, 5, 13, 1, 2, 10, 14, 7],
    [0, 12, 6, 4, 3, 13, 2, 14, 8, 11, 15, 9, 1, 5, 7, 10],
    [0, 13, 15, 10, 14, 6, 5, 8, 7, 9, 3, 12, 11, 1, 4, 2],
    [0, 14, 7, 11, 10, 8, 12, 2, 5, 15, 4, 3, 6, 13, 1, 9],
    [0, 15, 14, 5, 7, 3, 11, 4, 10, 13, 8, 6, 12, 9, 2, 1],
];
const POW_TABLE_2D: [[u8; 16]; 16] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    [0, 2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9, 1],
    [0, 3, 5, 15, 2, 6, 10, 13, 4, 12, 7, 9, 8, 11, 14, 1],
    [0, 4, 3, 12, 5, 7, 15, 9, 2, 8, 6, 11, 10, 14, 13, 1],
    [0, 5, 2, 10, 4, 7, 8, 14, 3, 15, 6, 13, 12, 9, 11, 1],
    [0, 6, 7, 1, 6, 7, 1, 6, 7, 1, 6, 7, 1, 6, 7, 1],
    [0, 7, 6, 1, 7, 6, 1, 7, 6, 1, 7, 6, 1, 7, 6, 1],
    [0, 8, 12, 10, 15, 1, 8, 12, 10, 15, 1, 8, 12, 10, 15, 1],
    [0, 9, 13, 15, 14, 7, 10, 5, 11, 12, 6, 3, 8, 4, 2, 1],
    [0, 10, 8, 15, 12, 1, 10, 8, 15, 12, 1, 10, 8, 15, 12, 1],
    [0, 11, 9, 12, 13, 6, 15, 3, 14, 8, 7, 4, 10, 2, 5, 1],
    [0, 12, 15, 8, 10, 1, 12, 15, 8, 10, 1, 12, 15, 8, 10, 1],
    [0, 13, 14, 10, 11, 6, 8, 2, 9, 15, 7, 5, 12, 3, 4, 1],
    [0, 14, 11, 8, 9, 7, 12, 4, 13, 10, 6, 2, 15, 5, 3, 1],
    [0, 15, 10, 12, 8, 1, 15, 10, 12, 8, 1, 15, 10, 12, 8, 1],
];
#[allow(dead_code)]
const EXP_TABLE: [u8; 16] = [1, 2, 4, 8, 3, 6, 12, 11, 5, 10, 7, 14, 15, 13, 9, 0];
#[allow(dead_code)]
const LOG_TABLE: [u8; 16] = [15, 0, 1, 4, 2, 8, 5, 10, 3, 14, 9, 7, 6, 13, 11, 12];

/// Implementation of GF(16) with P(x) = x4 + x + 1
/// The `inner` attribute only hold a single 4-bit number, even though it could hold two.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GaloisField2p4 {
    pub inner: u8,
}

impl Add for GaloisField2p4 {
    type Output = GaloisField2p4;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: Self) -> Self::Output {
        let inner = self.inner ^ rhs.inner;
        GaloisField2p4 { inner }
    }
}
impl AddAssign for GaloisField2p4 {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn add_assign(&mut self, rhs: Self) {
        self.inner ^= rhs.inner;
    }
}
impl Sub for GaloisField2p4 {
    type Output = GaloisField2p4;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn sub(self, rhs: Self) -> Self::Output {
        let inner = self.inner ^ rhs.inner;
        GaloisField2p4 { inner }
    }
}
impl SubAssign for GaloisField2p4 {
    #[allow(clippy::suspicious_op_assign_impl)]
    fn sub_assign(&mut self, rhs: Self) {
        self.inner ^= rhs.inner;
    }
}
impl Mul for GaloisField2p4 {
    type Output = GaloisField2p4;

    fn mul(self, rhs: Self) -> Self::Output {
        let inner = MUL_TABLE_2D[self.inner as usize][rhs.inner as usize];
        GaloisField2p4 { inner }
    }
}
impl MulAssign for GaloisField2p4 {
    fn mul_assign(&mut self, rhs: Self) {
        self.inner = MUL_TABLE_2D[self.inner as usize][rhs.inner as usize];
    }
}
impl Div for GaloisField2p4 {
    type Output = GaloisField2p4;

    fn div(self, rhs: Self) -> Self::Output {
        let inner = DIV_TABLE_2D[self.inner as usize][rhs.inner as usize];
        GaloisField2p4 { inner }
    }
}
impl DivAssign for GaloisField2p4 {
    fn div_assign(&mut self, rhs: Self) {
        self.inner = DIV_TABLE_2D[self.inner as usize][rhs.inner as usize];
    }
}
impl From<u8> for GaloisField2p4 {
    /// Creating GaloisField2p4 from u8 shifts the value down by 4 bits if it is too big to fit in.
    fn from(value: u8) -> Self {
        if value <= 0xF {
            GaloisField2p4 { inner: value }
        } else {
            GaloisField2p4 {
                inner: (value & 0xF0) >> 4,
            }
        }
    }
}

// Trait `Distribution<GaloisField2p4>` is not implemented for `StandardUniform`
impl Distribution<GaloisField2p4> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> GaloisField2p4 {
        let range: [GaloisField2p4; 15] = from_fn(|idx| GaloisField2p4::from((idx + 1) as u8));
        *range.choose(rng).unwrap()
    }
}

impl GaloisField2p4 {
    pub fn pow(self, exp: GaloisField2p4) -> GaloisField2p4 {
        let inner = POW_TABLE_2D[self.inner as usize][exp.inner as usize];
        GaloisField2p4 { inner }
    }
    pub fn pow_assign(&mut self, exp: GaloisField2p4) {
        self.inner = POW_TABLE_2D[self.inner as usize][exp.inner as usize];
    }
    pub const ZERO: GaloisField2p4 = GaloisField2p4 { inner: 0 };
    pub const ONE: GaloisField2p4 = GaloisField2p4 { inner: 1 };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::iter::repeat_n;
    #[test]
    #[ignore]
    fn generate_pow_table() {
        let mut table = [[0u8; 16]; 16];
        for x in 1..16u8 {
            for y in 1..16u8 {
                let n = repeat_n(x, y as usize)
                    .reduce(|a, b| MUL_TABLE_2D[a as usize][b as usize])
                    .unwrap();
                table[x as usize][y as usize] = n;
            }
        }
        for x in table {
            println!("{x:?},");
        }
    }
}
