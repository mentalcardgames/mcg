use crate::FRAGMENTS_PER_EPOCH;
use crate::network_coding::{Equation, GaloisField2p4};

#[derive(Default, Clone)]
pub struct Matrix {
    pub inner: Vec<Equation>,
}

impl Matrix {
    pub fn find_pivot(&self, column: usize, start: usize) -> Option<usize> {
        (start..self.inner.len()).find(|&i| self.inner[i].factors[column] != GaloisField2p4::ZERO)
    }

    /// Matrix elimination from https://docs.rs/gauss_jordan_elimination/0.2.0/src/gauss_jordan_elimination/lib.rs.html#23
    pub fn matrix_elimination(&mut self) {
        let n_rows = self.inner.len();
        // Eliminate lower left triangle from matrix
        let mut pivot_counter = 0;
        for column_idx in 0..FRAGMENTS_PER_EPOCH {
            if pivot_counter == n_rows {
                break;
            }
            match self.find_pivot(column_idx, pivot_counter) {
                None => {}
                Some(pivot_row_idx) => {
                    // Normalize the pivot to get identity
                    let denominator = self.inner[pivot_row_idx].factors[column_idx];
                    if denominator != GaloisField2p4::ONE {
                        self.inner[pivot_row_idx] /= denominator;
                    } else if denominator == GaloisField2p4::ZERO {
                        unreachable!("This shouldn't happen!");
                    }

                    // Subtract pivot row from all rows that are below it
                    for row in pivot_row_idx + 1..n_rows {
                        let factor = self.inner[row].factors[column_idx];
                        if factor == GaloisField2p4::ZERO {
                            continue;
                        }
                        let (pivot_slice, destination_slice) = self.inner.split_at_mut(row);
                        destination_slice[0] -= pivot_slice[pivot_row_idx].clone() * factor;
                    }

                    // Move pivot to row column, in order to get a "real" echelon form.
                    if pivot_counter != pivot_row_idx {
                        self.inner.swap(pivot_row_idx, pivot_counter);
                    }

                    pivot_counter += 1;
                }
            }
        }

        // Elimination of upper right triangle
        let mut pivot_counter = self.inner.len() - 1;
        for column_idx in (0..FRAGMENTS_PER_EPOCH).rev() {
            if pivot_counter == 0 {
                break;
            }
            match self.find_pivot(column_idx, pivot_counter) {
                None => {}
                Some(pivot_row_idx) => {
                    pivot_counter -= 1;

                    let pivot = self.inner[pivot_row_idx].factors[column_idx];
                    if pivot != GaloisField2p4::ONE {
                        // In case the pivot is not one we make it
                        self.inner[pivot_row_idx] /= pivot;
                    } else if pivot == GaloisField2p4::ZERO {
                        unreachable!("This shouldn't happen!");
                    }

                    // Subtract pivot row from all rows that are above it
                    for row_idx in 0..pivot_row_idx {
                        let factor = self.inner[row_idx].factors[column_idx];
                        if factor != GaloisField2p4::ZERO {
                            let (destination_slice, pivot_slice) =
                                self.inner.split_at_mut(pivot_row_idx);
                            destination_slice[row_idx] -= pivot_slice[0].clone() * factor;
                        }
                    }
                }
            }
        }
    }
    pub fn sweep_downwards(&mut self) {}
    pub fn sweep_upwards(&mut self) {}
}
