use crate::data_structures::Fragment;
use crate::{AP_LENGTH_INDEX_SIZE_BYTES, AP_MAX_SIZE_BYTES, FRAGMENT_SIZE_BYTES};
use std::cmp::min;
use std::io::Read;

#[derive(Clone)]
pub struct Package {
    pub size: u32,
    pub data: Vec<u8>,
}

impl Package {
    pub fn new(package: &[u8]) -> Self {
        if package.len() > AP_MAX_SIZE_BYTES {
            panic!("AP is too large");
        }
        let mut data = package.to_vec();
        data.shrink_to_fit();
        let size = data.len() as u32;
        Package { size, data }
    }
    pub fn from_read(mut package: impl Read) -> Self {
        let mut buf = Vec::new();
        package
            .read_to_end(&mut buf)
            .expect("Unable to read package!");
        Package::new(&buf)
    }
    pub fn into_fragments(self) -> Vec<Fragment> {
        debug_assert!(AP_LENGTH_INDEX_SIZE_BYTES <= size_of::<u32>());
        let Package { size, mut data } = self;
        let mut fragments = Vec::new();
        let mut first_fragment = [0u8; FRAGMENT_SIZE_BYTES];
        first_fragment[..size_of::<u32>()].copy_from_slice(&size.to_le_bytes());
        let end = min(data.len(), FRAGMENT_SIZE_BYTES - AP_LENGTH_INDEX_SIZE_BYTES);
        let first_data: Vec<u8> = data.drain(0..end).collect();
        first_fragment[AP_LENGTH_INDEX_SIZE_BYTES..end + AP_LENGTH_INDEX_SIZE_BYTES]
            .copy_from_slice(&first_data);
        fragments.push(first_fragment.into());
        while !data.is_empty() {
            let end = min(data.len(), FRAGMENT_SIZE_BYTES);
            let mut fragment = [0u8; FRAGMENT_SIZE_BYTES];
            let data: Vec<u8> = data.drain(..end).collect();
            fragment[..end].copy_from_slice(&data);
            fragments.push(fragment.into());
        }
        fragments
    }
    pub fn from_fragments(fragments: &[Fragment]) -> Self {
        debug_assert!(AP_LENGTH_INDEX_SIZE_BYTES <= size_of::<u32>());
        assert!(!fragments.is_empty());
        let mut size = [0; size_of::<u32>()];
        size[..AP_LENGTH_INDEX_SIZE_BYTES]
            .copy_from_slice(&fragments[0][..AP_LENGTH_INDEX_SIZE_BYTES]);
        let size = u32::from_le_bytes(size);
        let mut data = Vec::with_capacity(size as usize);
        let end = min(
            size as usize,
            FRAGMENT_SIZE_BYTES - AP_LENGTH_INDEX_SIZE_BYTES,
        );
        data.extend_from_slice(
            &fragments[0][AP_LENGTH_INDEX_SIZE_BYTES..end + AP_LENGTH_INDEX_SIZE_BYTES],
        );
        let mut fragment_idx = 1;
        while let Some(fragment) = fragments.get(fragment_idx) {
            let end = min(size as usize - data.len(), FRAGMENT_SIZE_BYTES);
            data.extend_from_slice(&fragment[..end]);
            fragment_idx += 1;
        }
        Package::new(&data)
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structures::Package;
    use crate::{AP_LENGTH_INDEX_SIZE_BYTES, FRAGMENT_SIZE_BYTES};
    use std::array::from_fn;

    #[test]
    fn into_fragments_test_0() {
        const PRIME_LEN: usize = 54;
        let primes: [u8; PRIME_LEN] = [
            2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83,
            89, 97, 101, 103, 107, 109, 113, 127, 131, 137, 139, 149, 151, 157, 163, 167, 173, 179,
            181, 191, 193, 197, 199, 211, 223, 227, 229, 233, 239, 241, 251,
        ];
        let package = Package::new(&primes);
        assert_eq!(package.size, PRIME_LEN as u32);
        let fragments = package.into_fragments();
        assert_eq!(fragments.len(), 1);
        assert_eq!(
            fragments[0][AP_LENGTH_INDEX_SIZE_BYTES..PRIME_LEN + AP_LENGTH_INDEX_SIZE_BYTES],
            primes
        );
        assert_eq!(
            fragments[0][PRIME_LEN + AP_LENGTH_INDEX_SIZE_BYTES..],
            [0; FRAGMENT_SIZE_BYTES - PRIME_LEN - AP_LENGTH_INDEX_SIZE_BYTES]
        );
    }
    #[test]
    fn into_fragments_test_1() {
        const DATA_LEN: usize = 1021;
        let data: [u8; DATA_LEN] = from_fn(|x| x as u8);
        let package = Package::new(&data);
        assert_eq!(package.size, DATA_LEN as u32);
        let fragments = package.into_fragments();
        assert_eq!(fragments.len(), 1);
        assert_eq!(
            fragments[0][AP_LENGTH_INDEX_SIZE_BYTES..DATA_LEN + AP_LENGTH_INDEX_SIZE_BYTES],
            data
        );
        assert_eq!(
            fragments[0][DATA_LEN + AP_LENGTH_INDEX_SIZE_BYTES..],
            [0; FRAGMENT_SIZE_BYTES - DATA_LEN - AP_LENGTH_INDEX_SIZE_BYTES]
        );
    }
    #[test]
    fn into_fragments_test_2() {
        const DATA_LEN: usize = 1024;
        let data: [u8; DATA_LEN] = from_fn(|x| x as u8);
        let package = Package::new(&data);
        assert_eq!(package.size, DATA_LEN as u32);
        let fragments = package.into_fragments();
        assert_eq!(fragments.len(), 2);
        assert_eq!(
            fragments[0][AP_LENGTH_INDEX_SIZE_BYTES..],
            data[..DATA_LEN - AP_LENGTH_INDEX_SIZE_BYTES]
        );
        assert_eq!(
            fragments[1][..AP_LENGTH_INDEX_SIZE_BYTES],
            data[FRAGMENT_SIZE_BYTES - AP_LENGTH_INDEX_SIZE_BYTES..]
        );
        assert_eq!(
            fragments[1][AP_LENGTH_INDEX_SIZE_BYTES..],
            [0; FRAGMENT_SIZE_BYTES - AP_LENGTH_INDEX_SIZE_BYTES]
        );
    }
    #[test]
    fn from_fragments_test_0() {
        const DATA_LEN: usize = 9001;
        let data: [u8; DATA_LEN] = from_fn(|x| x as u8);
        let package = Package::new(&data);
        assert_eq!(package.size, DATA_LEN as u32);
        let fragments = package.into_fragments();
        assert_eq!(fragments.len(), 9);
        let new = Package::from_fragments(fragments.clone().as_ref()).into_fragments();
        for (idx, frag) in fragments.iter().enumerate() {
            assert_eq!(frag, &new[idx]);
        }
    }
}
