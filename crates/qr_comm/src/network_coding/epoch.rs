use crate::data_structures::{Fragment, Frame, FrameFactor, FrameHeader, Package, WideFactor};
use crate::network_coding::{Equation, GaloisField2p4};
use crate::{
    AP_LENGTH_INDEX_SIZE_BYTES, BYTES_PER_PARTICIPANT, CODING_FACTORS_PER_PARTICIPANT_PER_FRAME,
    FRAGMENT_SIZE_BYTES, FRAGMENTS_PER_EPOCH, FRAGMENTS_PER_PARTICIPANT_PER_EPOCH,
    MAX_PARTICIPANTS,
};
use rand::random;
use std::array::from_fn;
use std::num::NonZeroU8;
use std::ops::Range;

pub struct Epoch {
    pub equations: Vec<Equation>,
    pub decoded_fragments: Vec<Vec<Fragment>>,
    pub meta_ap_fragments: Vec<Vec<Range<usize>>>,
    pub current_utilization: Box<[usize; FRAGMENTS_PER_EPOCH]>,
    pub elimination_flag: bool,
    pub header: FrameHeader,
}

impl Default for Epoch {
    fn default() -> Self {
        let equations = Vec::new();
        let decoded_fragments = from_fn::<_, MAX_PARTICIPANTS, _>(|_| {
            Vec::with_capacity(FRAGMENTS_PER_PARTICIPANT_PER_EPOCH)
        })
        .to_vec();
        let meta_ap_fragments = from_fn::<_, MAX_PARTICIPANTS, _>(|_| Vec::new()).to_vec();
        let current_utilization: Box<[usize; FRAGMENTS_PER_EPOCH]> = vec![0; FRAGMENTS_PER_EPOCH]
            .try_into()
            .expect("Error allocating memory!");
        let elimination_flag = false;
        let header = FrameHeader::default();
        Epoch {
            equations,
            decoded_fragments,
            meta_ap_fragments,
            current_utilization,
            elimination_flag,
            header,
        }
    }
}

impl Epoch {
    pub fn new(header: FrameHeader) -> Self {
        Self {
            header,
            ..Default::default()
        }
    }
    pub fn push_frame(&mut self, frame: Frame) {
        let Frame {
            factors,
            fragment,
            header: _header,
        } = frame;
        let factors: WideFactor = factors.into();
        let equation = Equation::new(factors, fragment);
        let utilization = equation.factors.utilized_fragments();

        // Check for new fragments this frame
        for (current, u) in self.current_utilization.iter_mut().zip(utilization.iter()) {
            let u = if *u { 1 } else { 0 };
            if *current == 0 && u == 1 {
                self.elimination_flag = true;
            }
            *current += u;
        }

        if self.elimination_flag {
            self.equations.push(equation);

            // Calculate how many equations are needed to solve new AP
            let number_equations = self
                .current_utilization
                .iter()
                .enumerate()
                .filter(|(idx, u)| {
                    let participant_idx = idx / FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
                    let fragment_idx = idx % FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
                    **u > 0
                        && self
                            .decoded_fragments
                            .get(participant_idx)
                            .map(|fragments| fragments.get(fragment_idx).is_some())
                            .unwrap_or(false)
                })
                .count();

            if self.equations.len() >= number_equations {
                let mut matrix = Vec::new();

                // Map encoded fragments into equations
                for (idx_participant, fragments) in self.decoded_fragments.iter().enumerate() {
                    for (idx_fragment, fragment) in fragments.iter().enumerate() {
                        let mut factors = WideFactor::default();
                        let idx =
                            idx_participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + idx_fragment;
                        factors[idx] = GaloisField2p4::ONE;
                        let equation = Equation::new(factors, fragment.clone());
                        matrix.push(equation);
                    }
                }
                matrix.append(self.equations.clone().as_mut());

                matrix_elimination(&mut matrix);

                // Append decoded fragments
                if matrix.iter().filter(|eq| eq.factors.is_plain()).count() == number_equations {
                    self.elimination_flag = false;
                    for eq in matrix {
                        let eq_idx = eq
                            .factors
                            .iter()
                            .enumerate()
                            .find(|(_idx, f)| **f != GaloisField2p4::ZERO)
                            .unwrap()
                            .0;
                        let participant_idx = eq_idx / CODING_FACTORS_PER_PARTICIPANT_PER_FRAME;
                        let fragment_idx = eq_idx % CODING_FACTORS_PER_PARTICIPANT_PER_FRAME;
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
    pub fn pop_frame(&self) -> Frame {
        // TODO think about how frames should pick their window widths
        let _width = [16; MAX_PARTICIPANTS];

        // Get a linear combination of frames that haven't been decoded yet
        let mut equation = self
            .equations
            .iter()
            .cloned()
            .fold(Equation::default(), |acc, new| acc + (new * random::<u8>()));

        // Add all fragments that are decoded
        for (participant_idx, fragments) in self.decoded_fragments.iter().enumerate() {
            for (fragment_idx, fragment) in fragments.iter().enumerate() {
                let mut factors = WideFactor::default();
                factors
                    [participant_idx * CODING_FACTORS_PER_PARTICIPANT_PER_FRAME + fragment_idx] =
                    GaloisField2p4::ONE;
                let eq = Equation::new(factors, fragment.clone());
                let factor: NonZeroU8 = random();
                equation += eq * u8::from(factor);
            }
        }
        let Equation { factors, fragment } = equation;
        let header = self.header;
        let (width, offsets) = factors.get_width_and_offsets();
        let mut frame_factors = Vec::new();
        for participant in 0..MAX_PARTICIPANTS {
            let start =
                participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + (offsets[participant] as usize);
            let stop = start + (2 * width[participant] as usize);
            let mut f = factors.inner[start..stop].to_vec();
            frame_factors.append(f.as_mut());
        }
        let frame_factors = from_fn(|idx| {
            frame_factors
                .get(idx)
                .unwrap_or(&GaloisField2p4::ZERO)
                .to_owned()
        });

        let coding_factors = FrameFactor::new(frame_factors, width, offsets);
        Frame::new(coding_factors, fragment, header)
    }
    pub fn write(&mut self, ap: Package) {
        if (ap.size as usize
            + self.decoded_fragments[self.header.participant as usize].len() * FRAGMENT_SIZE_BYTES)
            <= BYTES_PER_PARTICIPANT
        {
            let fragments = ap.into_fragments();
            let start = self.decoded_fragments[self.header.participant as usize].len();
            let end = start + fragments.len();
            let ap_info = Range { start, end };
            self.decoded_fragments[self.header.participant as usize].extend(fragments);
            self.meta_ap_fragments[self.header.participant as usize].push(ap_info);
        }
    }
    pub fn get_package(&self, participant: usize, index: usize) -> Option<Package> {
        if self.decoded_fragments[participant].is_empty() {
            return None;
        }
        if let Some(range) = self.meta_ap_fragments[participant].get(index) {
            return Some(Package::from_fragments(
                &self.decoded_fragments[participant][range.start..range.end],
            ));
        }
        let mut package = None;
        let mut fragment_index = 0;
        let mut package_index = -1;
        let mut number_used_fragments = 0;
        while package_index < index as isize {
            // TODO use last element from before index from self.meta_ap_fragments[participant] to start
            let mut size = [0; 4];
            let fragment = self.decoded_fragments[participant].get(fragment_index)?;
            size[..AP_LENGTH_INDEX_SIZE_BYTES]
                .copy_from_slice(&fragment[..AP_LENGTH_INDEX_SIZE_BYTES]);
            let size = u32::from_le_bytes(size);
            number_used_fragments =
                (size as usize + AP_LENGTH_INDEX_SIZE_BYTES).div_ceil(FRAGMENT_SIZE_BYTES);
            fragment_index += number_used_fragments;
            package_index += 1;
            // TODO add this range to self.meta_ap_fragments[participant] if its not inside
        }
        if fragment_index <= self.decoded_fragments[participant].len() {
            let start = fragment_index - number_used_fragments;
            let stop = fragment_index;
            package.replace(Package::from_fragments(
                &self.decoded_fragments[participant][start..stop],
            ));
            // TODO add range to self.meta_ap_fragments[participant]
        }
        package
    }
}

fn find_pivot(matrix: &[Equation], column: usize, start: usize) -> Option<usize> {
    (start..matrix.len()).find(|&i| matrix[i].factors[column] != GaloisField2p4::ZERO)
}

/// Matrix elimination from https://docs.rs/gauss_jordan_elimination/0.2.0/src/gauss_jordan_elimination/lib.rs.html#23
fn matrix_elimination(matrix: &mut [Equation]) {
    let n_rows = matrix.len();
    // Eliminate lower left triangle from matrix
    let mut pivot_counter = 0;
    for column_idx in 0..FRAGMENTS_PER_EPOCH {
        if pivot_counter == n_rows {
            break;
        }
        match find_pivot(matrix, column_idx, pivot_counter) {
            None => {}
            Some(pivot_row_idx) => {
                // Normalize the pivot to get identity
                let denominator = matrix[pivot_row_idx].factors[column_idx];
                if denominator != GaloisField2p4::ONE {
                    matrix[pivot_row_idx] /= denominator;
                } else if denominator == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are below it
                for row in pivot_row_idx + 1..n_rows {
                    let factor = matrix[row].factors[column_idx];
                    if factor == GaloisField2p4::ZERO {
                        continue;
                    }
                    let (pivot_slice, destination_slice) = matrix.split_at_mut(row);
                    destination_slice[0] -= pivot_slice[pivot_row_idx].clone() * factor;
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
    for column_idx in (0..FRAGMENTS_PER_EPOCH).rev() {
        if pivot_counter == 0 {
            break;
        }
        match find_pivot(matrix, column_idx, pivot_counter) {
            None => {}
            Some(pivot_row_idx) => {
                pivot_counter -= 1;

                let pivot = matrix[pivot_row_idx].factors[column_idx];
                if pivot != GaloisField2p4::ONE {
                    // In case the pivot is not one we make it
                    matrix[pivot_row_idx] /= pivot;
                } else if pivot == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are above it
                for row_idx in 0..pivot_row_idx {
                    let factor = matrix[row_idx].factors[column_idx];
                    if factor != GaloisField2p4::ZERO {
                        let (destination_slice, pivot_slice) = matrix.split_at_mut(pivot_row_idx);
                        destination_slice[row_idx] -= pivot_slice[0].clone() * factor;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
    use crate::data_structures::{Package, WideFactor};
    use crate::network_coding::epoch::matrix_elimination;
    use crate::network_coding::{Epoch, Equation, GaloisField2p4};
    use rand::random;
    use std::fs::File;

    #[test]
    fn get_package_test_0() {
        let mut e = Epoch::default();
        let file_0 = File::open("tests/data_0.txt").unwrap();
        let file_1 = File::open("tests/data_1.txt").unwrap();
        let package_0 = Package::from_read(&file_0);
        let package_1 = Package::from_read(&file_1);
        e.write(package_0.clone());
        e.write(package_1.clone());
        assert_eq!(e.get_package(0, 0).unwrap(), package_0);
        assert_eq!(e.get_package(0, 1).unwrap(), package_1);
        assert!(e.get_package(0, 2).is_none());
        assert!(e.get_package(1, 0).is_none());
    }
    #[test]
    fn get_package_test_1() {
        todo!("Test get_package(...) after receiving frames from a different epoch.");
    }
    #[test]
    fn matrix_elimination_test_0() {
        let file_0 = File::open("tests/data_0.txt").unwrap();
        let fragments = Package::from_read(file_0).into_fragments();
        let equations: Vec<Equation> = fragments
            .iter()
            .enumerate()
            .map(|(idx, frag)| {
                let mut factor = WideFactor::default();
                factor[idx + FRAGMENTS_PER_PARTICIPANT_PER_EPOCH] = GaloisField2p4::ONE;
                Equation::new(factor, frag.clone())
            })
            .collect();
        let mut matrix: Vec<Equation> = Vec::new();
        for _ in 0..equations.len() {
            let eq = equations
                .iter()
                .cloned()
                .fold(Equation::default(), |acc, e| {
                    acc + (e * (random::<u8>() & 0xF))
                });
            matrix.push(eq);
        }
        matrix_elimination(&mut matrix);
        for (idx, eq) in matrix.iter().enumerate() {
            assert!(eq.factors.is_plain());
            assert_eq!(eq.fragment, fragments[idx]);
        }
    }
    #[test]
    fn matrix_elimination_test_1() {}
    #[test]
    fn richtige_simulation() {
        todo!("Wie sieht eine 'richtige' Simulation aus?")
    }
}
