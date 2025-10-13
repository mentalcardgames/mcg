use crate::data_structures::{CodingFactor, Equation, Fragment, Frame, FrameHeader, Package};
use crate::{
    BYTES_PER_PARTICIPANT, CODING_FACTORS_SIZE, FRAGMENT_SIZE_BYTES, FRAGMENTS_PER_PARTICIPANT,
    MAX_PARTICIPANTS,
};
use galois_2p8::PrimitivePolynomialField;
use galois_2p8::field::PRIMITIVES;
use rand::random;
use std::array::from_fn;
use std::num::NonZeroU8;

#[allow(dead_code)]
pub struct Epoch {
    pub equations: Vec<Equation>,
    pub decoded_fragments: Vec<Vec<Fragment>>,
    pub current_utilization: [usize; MAX_PARTICIPANTS],
    pub elimination_flag: bool,
    pub gf_field: PrimitivePolynomialField,
    pub header: FrameHeader,
}

impl Default for Epoch {
    fn default() -> Self {
        let equations = Vec::new();
        let decoded_fragments =
            from_fn::<_, MAX_PARTICIPANTS, _>(|_| Vec::with_capacity(FRAGMENTS_PER_PARTICIPANT))
                .to_vec();
        let current_utilization = [0; MAX_PARTICIPANTS];
        let elimination_flag = false;
        let gf_field = PrimitivePolynomialField::new(*PRIMITIVES.first().unwrap()).unwrap();
        let header = FrameHeader::default();
        Epoch {
            equations,
            decoded_fragments,
            current_utilization,
            elimination_flag,
            gf_field,
            header,
        }
    }
}

impl Epoch {
    #[allow(dead_code)]
    pub fn new(header: FrameHeader, gf_field: PrimitivePolynomialField) -> Self {
        Self {
            header,
            gf_field,
            ..Default::default()
        }
    }
    #[allow(dead_code)]
    pub fn push_frame(&mut self, frame: Frame) {
        let Frame {
            coding_factor,
            fragment,
            header: _header,
        } = frame;
        let equation = Equation::new(coding_factor, fragment);
        let utilization = equation.utilized_fragments();

        // Check for new fragments this frame
        self.current_utilization = from_fn(|idx| {
            if utilization[idx] > self.current_utilization[idx] {
                self.elimination_flag = true;
                utilization[idx]
            } else {
                self.current_utilization[idx]
            }
        });

        if self.elimination_flag {
            self.equations.push(equation);

            // Calculate how many equations are needed to solve new AP
            let number_equations_for_solvability: [usize; MAX_PARTICIPANTS] = from_fn(|idx| {
                self.current_utilization[idx].saturating_sub(self.decoded_fragments[idx].len())
            });

            if self.equations.len() >= number_equations_for_solvability.iter().sum() {
                let mut matrix = Vec::new();

                // Map encoded fragments into equations
                for (idx_participant, fragments) in self.decoded_fragments.iter().enumerate() {
                    for (idx_fragment, fragment) in fragments.iter().enumerate() {
                        let idx = idx_participant * FRAGMENTS_PER_PARTICIPANT + idx_fragment;
                        let mut coding_factor = [0; CODING_FACTORS_SIZE];
                        coding_factor[idx] = 1;
                        let coding_factor = coding_factor.into();
                        let equation = Equation::new(coding_factor, *fragment);
                        matrix.push(equation);
                    }
                }
                matrix.append(self.equations.clone().as_mut());

                matrix_elimination(&mut matrix, &self.gf_field);

                // Append decoded fragments
                if matrix.iter().filter(|eq| eq.is_plain()).count()
                    == self.current_utilization.iter().sum()
                {
                    self.elimination_flag = false;
                    for eq in matrix {
                        let eq_idx = eq
                            .coding_factor
                            .iter()
                            .enumerate()
                            .find(|(_idx, f)| **f != 0)
                            .unwrap()
                            .0;
                        let participant_idx = eq_idx / FRAGMENTS_PER_PARTICIPANT;
                        let fragment_idx = eq_idx % FRAGMENTS_PER_PARTICIPANT;
                        if fragment_idx < self.decoded_fragments[participant_idx].len() {
                            continue;
                        }
                        self.decoded_fragments[participant_idx].push(eq.fragment);
                    }
                    self.equations = Vec::new();
                }
            }
        }
    }
    #[allow(dead_code)]
    pub fn pop_frame(&self) -> Frame {
        // Get a linear combination of frames that haven't been decoded yet
        let mut equation =
            self.equations
                .iter()
                .cloned()
                .fold(Equation::default(), |mut acc, new| {
                    acc.add_scaled_assign(random(), &new, &self.gf_field);
                    acc
                });

        // Add all fragments that are decoded
        for (participant_idx, fragments) in self.decoded_fragments.iter().enumerate() {
            for (fragment_idx, fragment) in fragments.iter().enumerate() {
                let mut eq = Equation::new(CodingFactor::default(), *fragment);
                eq.coding_factor[participant_idx * FRAGMENTS_PER_PARTICIPANT + fragment_idx] = 1;
                let factor: NonZeroU8 = random();
                equation.add_scaled_assign(factor.into(), &eq, &self.gf_field);
            }
        }
        let Equation {
            coding_factor,
            fragment,
        } = equation;
        let header = self.header;
        Frame::new(coding_factor, fragment, header)
    }
    #[allow(dead_code)]
    pub fn write(&mut self, source: impl AsRef<[u8]>) {
        let source = source.as_ref();
        if !source.is_empty()
            && (source.len()
                + self.decoded_fragments[self.header.participant as usize].len()
                    * FRAGMENT_SIZE_BYTES)
                <= BYTES_PER_PARTICIPANT
        {
            let ap = Package::new(source);
            let fragments = ap.into_fragments();
            self.decoded_fragments[self.header.participant as usize].extend(fragments);
        }
    }
    #[allow(dead_code)]
    pub fn get_package(&self, participant: usize, index: usize) -> Option<Package> {
        if self.decoded_fragments[participant].is_empty() {
            return None;
        }
        let mut package = None;
        let mut fragment_index = 0;
        let mut package_index = -1;
        let mut number_used_fragments = 0;
        while package_index < index as isize {
            let mut size = [0; 4];
            size.copy_from_slice(&self.decoded_fragments[participant].get(fragment_index)?[0..4]);
            let size = u32::from_le_bytes(size);
            number_used_fragments = (size as usize + 4).div_ceil(FRAGMENT_SIZE_BYTES);
            fragment_index += number_used_fragments;
            package_index += 1;
        }
        if fragment_index <= self.decoded_fragments[participant].len() {
            package.replace(Package::from_fragments(
                &self.decoded_fragments[participant]
                    [fragment_index - number_used_fragments..fragment_index],
            ));
        }
        package
    }
}

fn find_pivot(matrix: &[Equation], column: usize, start: usize) -> Option<usize> {
    (start..matrix.len()).find(|&i| matrix[i].coding_factor[column] != 0)
}

/// Matrix elimination from https://docs.rs/gauss_jordan_elimination/0.2.0/src/gauss_jordan_elimination/lib.rs.html#23
fn matrix_elimination(matrix: &mut [Equation], field: &PrimitivePolynomialField) {
    let n_rows = matrix.len();
    // Eliminate lower left triangle from matrix
    let mut pivot_counter = 0;
    for column_idx in 0..CODING_FACTORS_SIZE {
        if pivot_counter == n_rows {
            break;
        }
        match find_pivot(matrix, column_idx, pivot_counter) {
            None => {}
            Some(pivot_row_idx) => {
                // Normalize the pivot to get identity
                let denominator = matrix[pivot_row_idx].coding_factor[column_idx];
                if denominator != 1 {
                    matrix[pivot_row_idx].div_assign(denominator, field);
                } else if denominator == 0 {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are below it
                for row in pivot_row_idx + 1..n_rows {
                    let factor = matrix[row].coding_factor[column_idx];
                    if factor == 0 {
                        continue;
                    }
                    let (pivot_slice, destination_slice) = matrix.split_at_mut(row);
                    destination_slice[0].sub_scaled_assign(
                        factor,
                        &pivot_slice[pivot_row_idx],
                        field,
                    );
                }

                // Move pivot to row column, in order to get a "real" echelon form.
                if pivot_counter != pivot_row_idx {
                    matrix.swap(pivot_row_idx, pivot_counter);
                }

                pivot_counter += 1;
            }
        }
    }

    // Elimination of upper right triangle
    let mut pivot_counter = matrix.len() - 1;
    for column_idx in (0..CODING_FACTORS_SIZE).rev() {
        if pivot_counter == 0 {
            break;
        }
        match find_pivot(matrix, column_idx, pivot_counter) {
            None => {}
            Some(pivot_row_idx) => {
                pivot_counter -= 1;

                let pivot = matrix[pivot_row_idx].coding_factor[column_idx];
                if pivot != 1 {
                    // In case the pivot is not one we make it
                    matrix[pivot_row_idx].div_assign(pivot, field);
                } else if pivot == 0 {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are above it
                for row_idx in 0..pivot_row_idx {
                    let factor = matrix[row_idx].coding_factor[column_idx];
                    if factor != 0 {
                        let (destination_slice, pivot_slice) = matrix.split_at_mut(pivot_row_idx);
                        destination_slice[row_idx].sub_scaled_assign(
                            factor,
                            &pivot_slice[0],
                            field,
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::network_coding::epoch::{Epoch, matrix_elimination};
    use crate::data_structures::{CodingFactor, Equation, Fragment, Frame, FrameHeader};
    use crate::{CODING_FACTORS_SIZE, FRAGMENT_SIZE_BYTES, FRAGMENTS_PER_PARTICIPANT, MAX_PARTICIPANTS, EPOCH_SIZE_FRAGMENTS};
    use galois_2p8::field::PRIMITIVES;
    use galois_2p8::{Field, PrimitivePolynomialField};
    use rand::random;
    use std::array::from_fn;

    #[test]
    fn matrix_elimination_test_0() {
        let gf_field = PrimitivePolynomialField::new(*PRIMITIVES.first().unwrap()).unwrap();
        let fragment_0: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| x as u8);
        let fragment_1: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| (x + 1) as u8);
        let fragment_2: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| (x + 2) as u8);
        let fragment_0: Fragment = fragment_0.into();
        let fragment_1: Fragment = fragment_1.into();
        let fragment_2: Fragment = fragment_2.into();
        let mut factor_0 = [0u8; CODING_FACTORS_SIZE];
        let mut factor_1 = [0u8; CODING_FACTORS_SIZE];
        let mut factor_2 = [0u8; CODING_FACTORS_SIZE];
        factor_0
            .iter_mut()
            .take(3)
            .for_each(|item| *item = random());
        factor_1
            .iter_mut()
            .take(3)
            .for_each(|item| *item = random());
        factor_2
            .iter_mut()
            .take(3)
            .for_each(|item| *item = random());
        let factor_0: CodingFactor = factor_0.into();
        let factor_1: CodingFactor = factor_1.into();
        let factor_2: CodingFactor = factor_2.into();
        let mut encoded_0 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_0, &fragment_0, factor_0[0]);
        gf_field.add_scaled_multiword(&mut encoded_0, &fragment_1, factor_0[1]);
        gf_field.add_scaled_multiword(&mut encoded_0, &fragment_2, factor_0[2]);
        let mut encoded_1 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_1, &fragment_0, factor_1[0]);
        gf_field.add_scaled_multiword(&mut encoded_1, &fragment_1, factor_1[1]);
        gf_field.add_scaled_multiword(&mut encoded_1, &fragment_2, factor_1[2]);
        let mut encoded_2 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_2, &fragment_0, factor_2[0]);
        gf_field.add_scaled_multiword(&mut encoded_2, &fragment_1, factor_2[1]);
        gf_field.add_scaled_multiword(&mut encoded_2, &fragment_2, factor_2[2]);
        let encoded_0: Fragment = encoded_0.into();
        let encoded_1: Fragment = encoded_1.into();
        let encoded_2: Fragment = encoded_2.into();
        let equation_0 = Equation::new(factor_0, encoded_0);
        let equation_1 = Equation::new(factor_1, encoded_1);
        let equation_2 = Equation::new(factor_2, encoded_2);
        let mut matrix = vec![equation_0, equation_1, equation_2];
        matrix_elimination(&mut matrix, &gf_field);
        let equation_2 = matrix.pop().unwrap();
        let equation_1 = matrix.pop().unwrap();
        let equation_0 = matrix.pop().unwrap();
        assert_eq!(equation_0.coding_factor[0], 1);
        assert_eq!(equation_1.coding_factor[1], 1);

        assert_eq!(equation_2.coding_factor[2], 1);

        assert_eq!(equation_2.coding_factor[0], 0);
        assert_eq!(equation_2.coding_factor[1], 0);

        assert_eq!(equation_1.coding_factor[0], 0);
        assert_eq!(equation_1.coding_factor[2], 0);

        assert_eq!(equation_0.coding_factor[1], 0);
        assert_eq!(equation_0.coding_factor[2], 0);

        assert_eq!(equation_0.fragment.as_ref(), fragment_0.as_ref());
        assert_eq!(equation_1.fragment.as_ref(), fragment_1.as_ref());
        assert_eq!(equation_2.fragment.as_ref(), fragment_2.as_ref());
    }
    #[test]
    fn matrix_elimination_test_1() {
        let gf_field = PrimitivePolynomialField::new(*PRIMITIVES.first().unwrap()).unwrap();
        let fragment_0: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| x as u8);
        let fragment_1: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| (x + 1) as u8);
        let fragment_0: Fragment = fragment_0.into();
        let fragment_1: Fragment = fragment_1.into();
        let mut factor_0 = [0u8; CODING_FACTORS_SIZE];
        let mut factor_1 = [0u8; CODING_FACTORS_SIZE];
        let mut factor_2 = [0u8; CODING_FACTORS_SIZE];
        factor_0[0] = 128;
        factor_0[1] = 2;
        factor_1[0] = 3;
        factor_1[1] = 4;
        factor_2[0] = 1;
        factor_2[FRAGMENTS_PER_PARTICIPANT] = 1;
        let factor_0: CodingFactor = factor_0.into();
        let factor_1: CodingFactor = factor_1.into();
        let factor_2: CodingFactor = factor_2.into();
        let mut encoded_0 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_0, &fragment_0, factor_0[0]);
        gf_field.add_scaled_multiword(&mut encoded_0, &fragment_1, factor_0[1]);
        let mut encoded_1 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_1, &fragment_0, factor_1[0]);
        gf_field.add_scaled_multiword(&mut encoded_1, &fragment_1, factor_1[1]);
        let mut encoded_2 = [0u8; FRAGMENT_SIZE_BYTES];
        gf_field.add_scaled_multiword(&mut encoded_2, &fragment_0, factor_2[0]);
        gf_field.add_scaled_multiword(
            &mut encoded_2,
            &fragment_0,
            factor_2[FRAGMENTS_PER_PARTICIPANT],
        );
        let encoded_0: Fragment = encoded_0.into();
        let encoded_1: Fragment = encoded_1.into();
        let encoded_2: Fragment = encoded_2.into();
        let equation_0 = Equation::new(factor_0, encoded_0);
        let equation_1 = Equation::new(factor_1, encoded_1);
        let equation_2 = Equation::new(factor_2, encoded_2);
        let mut matrix = vec![equation_0, equation_1, equation_2];
        matrix_elimination(&mut matrix, &gf_field);
        let equation_2 = matrix.pop().unwrap();
        let equation_1 = matrix.pop().unwrap();
        let equation_0 = matrix.pop().unwrap();
        assert_eq!(equation_0.coding_factor[0], 1);
        assert_eq!(equation_0.coding_factor[1], 0);
        assert_eq!(equation_1.coding_factor[0], 0);
        assert_eq!(equation_1.coding_factor[1], 1);

        assert_eq!(equation_2.coding_factor[0], 0);
        assert_eq!(equation_2.coding_factor[FRAGMENTS_PER_PARTICIPANT], 1);

        assert_eq!(equation_0.fragment.as_ref(), fragment_0.as_ref());
        assert_eq!(equation_1.fragment.as_ref(), fragment_1.as_ref());
        assert_eq!(equation_2.fragment.as_ref(), fragment_0.as_ref());
    }
    #[test]
    fn push_frame_test_0() {
        let fragment_0: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| x as u8);
        let fragment_1: [u8; FRAGMENT_SIZE_BYTES] = from_fn(|x| (x + 1) as u8);
        let fragment_0: Fragment = fragment_0.into();
        let fragment_1: Fragment = fragment_1.into();
        let mut factor_0 = [0u8; CODING_FACTORS_SIZE];
        let mut factor_1 = [0u8; CODING_FACTORS_SIZE];
        factor_0[FRAGMENTS_PER_PARTICIPANT] = 2;
        factor_0[FRAGMENTS_PER_PARTICIPANT + 1] = 5;
        factor_1[FRAGMENTS_PER_PARTICIPANT] = 8;
        factor_1[FRAGMENTS_PER_PARTICIPANT + 1] = 37;
        let factor_0: CodingFactor = factor_0.into();
        let factor_1: CodingFactor = factor_1.into();
        let header = FrameHeader::default();
        let mut epoch = Epoch::default();
        let mut encoded_0 = [0u8; FRAGMENT_SIZE_BYTES];
        epoch.gf_field.add_scaled_multiword(
            &mut encoded_0,
            &fragment_0,
            factor_0[FRAGMENTS_PER_PARTICIPANT],
        );
        epoch.gf_field.add_scaled_multiword(
            &mut encoded_0,
            &fragment_1,
            factor_0[FRAGMENTS_PER_PARTICIPANT + 1],
        );
        let mut encoded_1 = [0u8; FRAGMENT_SIZE_BYTES];
        epoch.gf_field.add_scaled_multiword(
            &mut encoded_1,
            &fragment_0,
            factor_1[FRAGMENTS_PER_PARTICIPANT],
        );
        epoch.gf_field.add_scaled_multiword(
            &mut encoded_1,
            &fragment_1,
            factor_1[FRAGMENTS_PER_PARTICIPANT + 1],
        );
        let encoded_0: Fragment = encoded_0.into();
        let encoded_1: Fragment = encoded_1.into();
        let frame_0 = Frame::new(factor_0, encoded_0, header);
        let frame_1 = Frame::new(factor_1, encoded_1, header);
        epoch.push_frame(frame_0);
        epoch.push_frame(frame_1);
        assert_eq!(epoch.decoded_fragments[1].len(), 2);
        assert_eq!(epoch.decoded_fragments[1][0].as_ref(), fragment_0.as_ref());
        assert_eq!(epoch.decoded_fragments[1][1].as_ref(), fragment_1.as_ref());
    }
    const DATA_0: &[u8; 12] = b"Hello World!";
    const DATA_1: &[u8; 80] =
    b"'May the force ever be in your favor, Mr. Potter' -Gandalf, Chronicles of Narnia";
    const DATA_2: &[u8; 1304] = b"Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet. Lorem ipsum dolor sit amet, consetetur sadipscing elitr, sed diam nonumy eirmod tempor invidunt ut labore et dolore magna aliquyam erat, sed diam voluptua. At vero eos et accusam et justo duo dolores et ea rebum. Stet clita kasd gubergren, no sea takimata sanctus est Lorem ipsum dolor sit amet.

Duis autem vel eum iriure dolor in hendrerit in vulputate velit esse molestie consequat, vel illum dolore eu feugiat nulla facilisis at vero eros et accumsan et iusto odio dignissim qui blandit praesent luptatum zzril delenit augue duis dolore te feugait nulla facilisi. Lorem ipsum dolor sit amet, consectetuer adipiscing elit, sed diam nonummy nibh euismod tincidunt ut laoreet dolore magna aliquam erat volutpat.";
    #[test]
    fn get_package_test_0() {
        let mut epoch = Epoch::default();
        epoch.write(DATA_0);
        epoch.write(DATA_1);
        let package_0 = epoch.get_package(0, 0);
        assert!(package_0.is_some());
        dbg!(String::from_utf8(package_0.clone().unwrap().data)).unwrap();
        assert_eq!(package_0.unwrap().data, DATA_0);
        let package_1 = epoch.get_package(0, 1);
        assert!(package_1.is_some());
        dbg!(String::from_utf8(package_1.clone().unwrap().data)).unwrap();
        assert_eq!(package_1.unwrap().data, DATA_1);
        assert!(epoch.get_package(0, 2).is_none());
    }
    #[test]
    fn get_package_test_1() {
        let mut epoch = Epoch::default();
        epoch.write(DATA_2);
        epoch.write(DATA_1);
        let package_0 = epoch.get_package(0, 0);
        assert!(package_0.is_some());
        println!(
            "{}",
            String::from_utf8(package_0.clone().unwrap().data).unwrap()
        );
        assert_eq!(package_0.unwrap().data, DATA_2);
        let package_1 = epoch.get_package(0, 1);
        assert!(package_1.is_some());
        println!(
            "{}",
            String::from_utf8(package_1.clone().unwrap().data).unwrap()
        );
        assert_eq!(package_1.unwrap().data, DATA_1);
        assert!(epoch.get_package(0, 2).is_none());
    }
    /// This test simulates the modified butterfly network with two source node s_1 & s_2,
    /// two intermediary i_1 & i_2 and two target nodes t_1 & t_2.
    #[test]
    fn modified_butterfly_network_test_0() {
        // Set up of participants
        let mut s_1 = Epoch::default();
        s_1.header.participant = 1;
        let mut s_2 = Epoch::default();
        s_2.header.participant = 2;
        let mut i_1 = Epoch::default();
        i_1.header.participant = 3;
        let mut i_2 = Epoch::default();
        i_2.header.participant = 4;
        let mut t_1 = Epoch::default();
        t_1.header.participant = 5;
        let mut t_2 = Epoch::default();
        t_2.header.participant = 6;

        // Give s_1 & s_2 their corresponding data
        s_1.write(DATA_0);
        s_1.write(DATA_1);
        s_2.write(DATA_2);

        // Simulate first time step
        let frame_s_1_0 = s_1.pop_frame();
        let frame_s_1_1 = s_1.pop_frame();
        let _frame_s_1_2 = s_1.pop_frame();
        let frame_s_1_3 = s_1.pop_frame();
        let frame_s_2_0 = s_2.pop_frame();
        let frame_s_2_1 = s_2.pop_frame();
        let _frame_s_2_2 = s_2.pop_frame();
        let frame_s_2_3 = s_2.pop_frame();
        i_1.push_frame(frame_s_1_0);
        i_1.push_frame(frame_s_2_0);
        assert_eq!(i_1.decoded_fragments[1].len(), 0);
        assert_eq!(i_1.decoded_fragments[2].len(), 0);
        assert_eq!(i_1.equations.len(), 2);

        // Simulate second time step
        let frame_i_1_0 = i_1.pop_frame();
        let frame_i_1_1 = i_1.pop_frame();
        i_1.push_frame(frame_s_1_1);
        i_1.push_frame(frame_s_2_1);
        assert_eq!(i_1.decoded_fragments[1].len(), 2);
        assert_eq!(i_1.decoded_fragments[2].len(), 2);
        assert_eq!(i_1.equations.len(), 0);

        let frame_i_1_2 = i_1.pop_frame();
        let _frame_i_1_3 = i_1.pop_frame();
        i_2.push_frame(frame_i_1_0);
        i_2.push_frame(frame_i_1_1);
        let frame_i_2_0 = i_2.pop_frame();
        i_2.push_frame(frame_i_1_2);
        // i_2.push_frame(frame_i_1_3);
        assert_eq!(i_2.decoded_fragments[1].len(), 0);
        assert_eq!(i_2.decoded_fragments[2].len(), 0);
        assert_eq!(i_2.equations.len(), 3);

        // Simulate last time step
        let frame_i_2_1 = i_2.pop_frame();
        t_1.push_frame(frame_s_1_1);
        t_1.push_frame(frame_i_2_0);
        t_1.push_frame(frame_i_2_1);
        // t_1.push_frame(frame_s_1_2);
        t_1.push_frame(frame_s_1_3);
        assert_eq!(t_1.decoded_fragments[1].len(), 2);
        assert_eq!(t_1.decoded_fragments[2].len(), 2);
        assert_eq!(t_1.equations.len(), 0);
        t_2.push_frame(frame_s_2_1);
        t_2.push_frame(frame_i_2_0);
        t_2.push_frame(frame_i_2_1);
        // t_2.push_frame(frame_s_2_2);
        t_2.push_frame(frame_s_2_3);
        assert_eq!(t_2.decoded_fragments[1].len(), 2);
        assert_eq!(t_2.decoded_fragments[2].len(), 2);
        assert_eq!(t_2.equations.len(), 0);
    }
    #[test]
    fn speed_test() {
        let mut generator = Epoch::default();
        const SIZE: usize = FRAGMENT_SIZE_BYTES * FRAGMENTS_PER_PARTICIPANT - 4;
        let data: [u8; SIZE] = from_fn(|x| x as u8);
        for idx in 0..MAX_PARTICIPANTS {
            generator.header.participant = idx as u8;
            generator.write(data);
        }
        for _ in 0..2 {
            let mut consumer = Epoch::default();
            for _ in 0..EPOCH_SIZE_FRAGMENTS {
                consumer.push_frame(generator.pop_frame());
            }
            assert!(consumer.equations.is_empty());
        }
    }
}
