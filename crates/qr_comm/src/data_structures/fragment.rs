use crate::FRAGMENT_SIZE_BYTES;
use std::ops::{Deref, DerefMut};

#[derive(Clone, PartialEq, Debug)]
pub struct Fragment {
    pub inner: Box<[u8; FRAGMENT_SIZE_BYTES]>,
}

impl Default for Fragment {
    fn default() -> Self {
        let inner = vec![0u8; FRAGMENT_SIZE_BYTES]
            .try_into()
            .expect("Error allocating memory!");
        Fragment { inner }
    }
}
impl Deref for Fragment {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner.deref()
    }
}
impl DerefMut for Fragment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner.deref_mut()
    }
}

impl Fragment {
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.iter().all(|&x| x == 0)
    }
}
