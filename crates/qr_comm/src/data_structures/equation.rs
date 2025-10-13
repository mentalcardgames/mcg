use crate::data_structures::{CodingFactor, Fragment};
use crate::{FRAGMENTS_PER_PARTICIPANT, MAX_PARTICIPANTS};
use galois_2p8::Field;

#[derive(Copy, Clone, Default)]
pub struct Equation {
    pub coding_factor: CodingFactor,
    pub fragment: Fragment,
}

impl Equation {
    pub fn new(coding_factor: CodingFactor, fragment: Fragment) -> Self {
        Equation {
            coding_factor,
            fragment,
        }
    }

    pub fn utilized_fragments(&self) -> [usize; MAX_PARTICIPANTS] {
        let mut utilization = [0; MAX_PARTICIPANTS];
        for (idx, participant) in utilization.iter_mut().enumerate() {
            let offset = idx * FRAGMENTS_PER_PARTICIPANT;
            for fragment in 0..FRAGMENTS_PER_PARTICIPANT {
                if self.coding_factor[offset + fragment] == 0 {
                    *participant = fragment;
                    break;
                }
            }
        }
        utilization
    }

    pub fn is_plain(&self) -> bool {
        self.coding_factor.iter().filter(|&f| *f != 0).count() == 1
    }
}

/// Mathematical operations
impl Equation {
    pub fn div_assign(&mut self, denominator: u8, field: &impl Field) {
        field.div_multiword(self.coding_factor.as_mut(), denominator);
        field.div_multiword(self.fragment.as_mut(), denominator);
    }
    pub fn mul_assign(&mut self, factor: u8, field: &impl Field) {
        field.mult_multiword(self.coding_factor.as_mut(), factor);
        field.mult_multiword(self.fragment.as_mut(), factor);
    }
    pub fn sub_scaled_assign(&mut self, scale: u8, rhs: &Self, field: &impl Field) {
        field.sub_scaled_multiword(&mut self.coding_factor, &rhs.coding_factor, scale);
        field.sub_scaled_multiword(&mut self.fragment, &rhs.fragment, scale);
    }
    pub fn add_scaled_assign(&mut self, scale: u8, rhs: &Self, field: &impl Field) {
        field.add_scaled_multiword(&mut self.coding_factor, &rhs.coding_factor, scale);
        field.add_scaled_multiword(&mut self.fragment, &rhs.fragment, scale);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CODING_FACTORS_SIZE;
    #[test]
    fn test_0_utilized_fragments() {
        let fragment = Fragment::default();
        let mut coding_factor = CodingFactor::new([0; CODING_FACTORS_SIZE]);
        for i in 0..4 {
            coding_factor[i] = 1;
        }
        let equation = Equation::new(coding_factor, fragment);
        let mut expected = [0; MAX_PARTICIPANTS];
        expected[0] = 4;
        assert_eq!(equation.utilized_fragments(), expected);
    }
    #[test]
    fn test_1_utilized_fragments() {
        let fragment = Fragment::default();
        let mut coding_factor = CodingFactor::new([0; CODING_FACTORS_SIZE]);
        for i in 0..4 {
            coding_factor[i] = 1;
        }
        for i in 0..5 {
            coding_factor[i + 2 * FRAGMENTS_PER_PARTICIPANT] = 1;
        }
        let equation = Equation::new(coding_factor, fragment);
        let mut expected = [0; MAX_PARTICIPANTS];
        expected[0] = 4;
        expected[2] = 5;
        assert_eq!(equation.utilized_fragments(), expected);
    }
}
