use crate::CODING_FACTORS_SIZE;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone)]
pub struct CodingFactor {
    pub inner: [u8; CODING_FACTORS_SIZE],
}

impl Default for CodingFactor {
    fn default() -> Self {
        let inner = [0; CODING_FACTORS_SIZE];
        Self { inner }
    }
}
impl Deref for CodingFactor {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.as_slice()
    }
}
impl DerefMut for CodingFactor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl CodingFactor {
    pub fn new(data: [u8; CODING_FACTORS_SIZE]) -> Self {
        debug_assert_eq!(CODING_FACTORS_SIZE, 688);
        CodingFactor { inner: data }
    }
}
