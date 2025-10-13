use crate::FRAGMENT_SIZE_BYTES;
use std::ops::{Deref, DerefMut};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Fragment {
    pub inner: [u8; FRAGMENT_SIZE_BYTES],
}

impl Default for Fragment {
    fn default() -> Self {
        Fragment {
            inner: [0; FRAGMENT_SIZE_BYTES],
        }
    }
}
impl Deref for Fragment {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl DerefMut for Fragment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl Fragment {
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.iter().all(|&x| x == 0)
    }
}
