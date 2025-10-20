use crate::data_structures::{
    Equation, Fragment, Frame, FrameFactor, FrameHeader, Package, WideFactor,
};
use crate::network_coding::GaloisField2p4;
use crate::{
    BYTES_PER_PARTICIPANT, CODING_FACTORS_PER_PARTICIPANT_PER_FRAME, CODING_FACTORS_SIZE_BYTES,
    FRAGMENTS_PER_EPOCH, FRAGMENTS_PER_PARTICIPANT_PER_EPOCH, FRAGMENT_SIZE_BYTES,
    MAX_PARTICIPANTS,
};
use rand::random;
use std::array::from_fn;
use std::num::NonZeroU8;

pub struct Epoch {
    pub equations: Vec<Equation>,
    pub decoded_fragments: Vec<Vec<Fragment>>,
    pub current_utilization: [usize; FRAGMENTS_PER_EPOCH],
    pub elimination_flag: bool,
    pub header: FrameHeader,
}

impl Default for Epoch {
    fn default() -> Self {
        let equations = Vec::new();
        let decoded_fragments = from_fn::<_, MAX_PARTICIPANTS, _>(|_| {
            Vec::with_capacity(CODING_FACTORS_PER_PARTICIPANT_PER_FRAME)
        })
        .to_vec();
        let current_utilization = [0; FRAGMENTS_PER_EPOCH];
        let elimination_flag = false;
        let header = FrameHeader::default();
        Epoch {
            equations,
            decoded_fragments,
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
            coding_factors,
            fragment,
            header: _header,
        } = frame;
        let factors: WideFactor = coding_factors.into();
        let equation = Equation::new(factors, fragment);
        let utilization = equation.utilized_fragments();

        // Check for new fragments this frame
        self.current_utilization = from_fn(|idx| {
            let n = self.current_utilization[idx];
            let u = if utilization[idx] { 1 } else { 0 };
            if n == 0 && u == 1 {
                self.elimination_flag = true;
            }
            n + u
        });

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
                        let equation = Equation::new(factors, *fragment);
                        matrix.push(equation);
                    }
                }
                matrix.append(self.equations.clone().as_mut());

                matrix_elimination(&mut matrix);

                // Append decoded fragments
                if matrix.iter().filter(|eq| eq.is_plain()).count() == number_equations {
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
    #[allow(dead_code)]
    pub fn pop_frame(&self) -> Frame {
        let _width = [16; MAX_PARTICIPANTS];

        // Get a linear combination of frames that haven't been decoded yet
        let mut equation =
            self.equations
                .iter()
                .cloned()
                .fold(Equation::default(), |mut acc, new| {
                    acc.add_scaled_assign(random(), &new);
                    acc
                });

        // Add all fragments that are decoded
        for (participant_idx, fragments) in self.decoded_fragments.iter().enumerate() {
            for (fragment_idx, fragment) in fragments.iter().enumerate() {
                let mut factors = WideFactor::default();
                factors
                    [participant_idx * CODING_FACTORS_PER_PARTICIPANT_PER_FRAME + fragment_idx] =
                    GaloisField2p4::ONE;
                let eq = Equation::new(factors, *fragment);
                let factor: NonZeroU8 = random();
                equation.add_scaled_assign(factor.into(), &eq);
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
    (start..matrix.len()).find(|&i| matrix[i].factors[column] != GaloisField2p4::ZERO)
}

/// Matrix elimination from https://docs.rs/gauss_jordan_elimination/0.2.0/src/gauss_jordan_elimination/lib.rs.html#23
fn matrix_elimination(matrix: &mut [Equation]) {
    let n_rows = matrix.len();
    // Eliminate lower left triangle from matrix
    let mut pivot_counter = 0;
    for column_idx in 0..CODING_FACTORS_SIZE_BYTES {
        if pivot_counter == n_rows {
            break;
        }
        match find_pivot(matrix, column_idx, pivot_counter) {
            None => {}
            Some(pivot_row_idx) => {
                // Normalize the pivot to get identity
                let denominator = matrix[pivot_row_idx].factors[column_idx];
                if denominator != GaloisField2p4::ONE {
                    matrix[pivot_row_idx].div_assign(denominator.inner);
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
                    destination_slice[0]
                        .sub_scaled_assign(factor.inner, &pivot_slice[pivot_row_idx]);
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
    for column_idx in (0..CODING_FACTORS_SIZE_BYTES).rev() {
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
                    matrix[pivot_row_idx].div_assign(pivot.inner);
                } else if pivot == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are above it
                for row_idx in 0..pivot_row_idx {
                    let factor = matrix[row_idx].factors[column_idx];
                    if factor != GaloisField2p4::ZERO {
                        let (destination_slice, pivot_slice) = matrix.split_at_mut(pivot_row_idx);
                        destination_slice[row_idx].sub_scaled_assign(factor.inner, &pivot_slice[0]);
                    }
                }
            }
        }
    }
}
