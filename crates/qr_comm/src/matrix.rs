use crate::FRAGMENTS_PER_EPOCH;
use crate::network_coding::{Equation, GaloisField2p4};

#[derive(Default, Clone)]
pub struct Matrix {
    pub inner: Vec<Equation>,
}

impl Matrix {
    pub fn find_pivot(&self, column: usize, start: usize) -> Option<usize> {
        (start..self.inner.len()).find(|&i| self.inner[i].factors.get(column) != GaloisField2p4::ZERO)
    }

    /// Matrix elimination from https://docs.rs/gauss_jordan_elimination/0.2.0/src/gauss_jordan_elimination/lib.rs.html#23
    pub fn matrix_elimination(&mut self) {
        self.sweep_downwards();
        self.sweep_upwards();
    }
    /// Eliminate the lower left triangle
    pub fn sweep_downwards(&mut self) {
        let mut pivot_counter = 0;
        for column_idx in 0..FRAGMENTS_PER_EPOCH { // TODO change to size of utilization
            if pivot_counter == self.inner.len() {
                break;
            }
            if let Some(pivot_row_idx) = self.find_pivot(column_idx, pivot_counter) {
                // Normalize the pivot to get identity
                let denominator = self.inner[pivot_row_idx].factors.get(column_idx);
                if denominator != GaloisField2p4::ONE {
                    self.inner[pivot_row_idx] /= denominator;
                } else if denominator == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are below it
                for row in pivot_row_idx + 1..self.inner.len() {
                    let factor = self.inner[row].factors.get(column_idx);
                    if factor == GaloisField2p4::ZERO {
                        continue;
                    }
                    let (pivot_slice, destination_slice) = self.inner.split_at_mut(row);
                    destination_slice[0] -= pivot_slice[pivot_row_idx].clone() * factor;
                    if self.inner[row].factors.is_zero() {
                        self.inner.remove(row);
                    }
                }

                // Move pivot to row column, in order to get a "real" echelon form.
                if pivot_counter != pivot_row_idx {
                    self.inner.swap(pivot_row_idx, pivot_counter);
                }

                pivot_counter += 1;
            }
        }
    }
    /// Eliminate only the lowest equation
    pub fn single_sweep_down(&mut self) {
        let mut pivot_counter: usize = 0;
        for column_idx in 0..FRAGMENTS_PER_EPOCH {
            if pivot_counter.saturating_add(1) == self.inner.len() {
                break;
            }
            if let Some(pivot_row_idx) = self.find_pivot(column_idx, pivot_counter) {
                // Normalize the pivot to get identity
                let denominator = self.inner[pivot_row_idx].factors.get(column_idx);
                if denominator != GaloisField2p4::ONE {
                    self.inner[pivot_row_idx] /= denominator;
                } else if denominator == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Only subtract pivot row from last row
                let row = self.inner.len() - 1;
                let factor = self.inner[row].factors.get(column_idx);
                if factor == GaloisField2p4::ZERO {
                    continue;
                }
                if let Some(pivot_eq) = self.inner.get(pivot_row_idx).cloned() {
                    self.inner[row] -= pivot_eq * factor;
                } else {
                    panic!("This should not panic!");
                }

                // Move pivot to row column, in order to get a "real" echelon form.
                if pivot_counter != pivot_row_idx {
                    self.inner.swap(pivot_row_idx, pivot_counter);
                }

                pivot_counter += 1;
            }
        }
    }
    /// Eliminate the upper right triangle
    pub fn sweep_upwards(&mut self) {
        let mut pivot_counter = self.inner.len() - 1;
        for column_idx in (0..FRAGMENTS_PER_EPOCH).rev() {
            if pivot_counter == 0 {
                break;
            }
            if let Some(pivot_row_idx) = self.find_pivot(column_idx, pivot_counter) {
                pivot_counter -= 1;

                let pivot = self.inner[pivot_row_idx].factors.get(column_idx);
                if pivot != GaloisField2p4::ONE {
                    // In case the pivot is not one we make it
                    self.inner[pivot_row_idx] /= pivot;
                } else if pivot == GaloisField2p4::ZERO {
                    unreachable!("This shouldn't happen!");
                }

                // Subtract pivot row from all rows that are above it
                for row_idx in 0..pivot_row_idx {
                    let factor = self.inner[row_idx].factors.get(column_idx);
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
