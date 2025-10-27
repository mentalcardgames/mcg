use crate::data_structures::{Fragment, Frame, FrameFactor, FrameHeader, Package, WideFactor};
use crate::matrix::Matrix;
use crate::network_coding::{Equation, GaloisField2p4};
use crate::{
    AP_LENGTH_INDEX_SIZE_BYTES, BYTES_PER_PARTICIPANT, CODING_FACTORS_PER_FRAME,
    CODING_FACTORS_PER_PARTICIPANT_PER_FRAME, FRAGMENTS_PER_EPOCH, FRAGMENTS_PER_PARTICIPANT_PER_EPOCH,
    FRAGMENT_SIZE_BYTES, MAX_PARTICIPANTS,
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
            // TODO Think about how the header should be used e.g. implement starting a new epoch
            header: _header,
        } = frame;
        let factors: WideFactor = factors.into();
        let equation = Equation::new(factors, fragment);
        let utilization: Box<[bool; FRAGMENTS_PER_EPOCH]> = equation.factors.utilized_fragments();

        // Check for new fragments this frame
        for (current, utilized) in self.current_utilization.iter_mut().zip(utilization.iter()) {
            let u = if *utilized { 1 } else { 0 };
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
                            .map(|fragments| fragments.get(fragment_idx).is_none())
                            .unwrap_or(true)
                })
                .count();

            if self.equations.len() >= number_equations {
                let mut matrix = Matrix::default();

                // Map encoded fragments into equations
                for (idx_participant, fragments) in self.decoded_fragments.iter().enumerate() {
                    for (idx_fragment, fragment) in fragments.iter().enumerate() {
                        let mut factors = WideFactor::default();
                        let idx =
                            idx_participant * FRAGMENTS_PER_PARTICIPANT_PER_EPOCH + idx_fragment;
                        factors[idx] = GaloisField2p4::ONE;
                        let equation = Equation::new(factors, fragment.clone());
                        matrix.inner.push(equation);
                    }
                }
                matrix.inner.append(self.equations.clone().as_mut());

                matrix.matrix_elimination();

                // Append decoded fragments
                if matrix
                    .inner
                    .iter()
                    .filter(|eq| eq.factors.is_plain())
                    .count()
                    == number_equations
                {
                    self.elimination_flag = false;
                    for eq in matrix.inner {
                        let eq_idx = eq
                            .factors
                            .iter()
                            .enumerate()
                            .find(|(_idx, f)| **f != GaloisField2p4::ZERO)
                            .unwrap()
                            .0;
                        let participant_idx = eq_idx / FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
                        let fragment_idx = eq_idx % FRAGMENTS_PER_PARTICIPANT_PER_EPOCH;
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
                let eq = Equation::plain_at_index(
                    participant_idx * CODING_FACTORS_PER_PARTICIPANT_PER_FRAME + fragment_idx,
                    fragment.clone(),
                );
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

        let coding_factors = FrameFactor::new(frame_factors, width, offsets)
            .expect("Looks like I did something wrong!");
        Frame::new(coding_factors, fragment, header)
    }
    pub fn pop_recent_frame(&self) -> Option<Frame> {
        let mut widths = [0u8; MAX_PARTICIPANTS];
        let mut sum_width = 0;
        let mut offsets = [0u16; MAX_PARTICIPANTS];
        let mut factors = [GaloisField2p4::ZERO; CODING_FACTORS_PER_FRAME];
        let mut coding_factor_idx = 0;
        let mut fragment = Fragment::default();
        for participant in 0..MAX_PARTICIPANTS {
            if let Some(Range { start, end }) = self.find_range_of_most_recent_package(participant)
            {
                let width = end - start;
                sum_width += width;
                if sum_width > CODING_FACTORS_PER_FRAME {
                    return None;
                }
                widths[participant] = width.div_ceil(2) as u8;
                offsets[participant] = start as u16;
                // TODO move this after participant loop to fill widths up to maximum
                for frag in &self.decoded_fragments[participant][start..end] {
                    let factor = random::<GaloisField2p4>();
                    factors[coding_factor_idx] = factor;
                    coding_factor_idx += 1;
                    fragment += frag.clone() * factor;
                }
            }
        }
        let factors = FrameFactor::new(factors, widths, offsets).unwrap();
        let header = self.header;
        let frame = Frame::new(factors, fragment, header);
        Some(frame)
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
    pub fn find_range_of_most_recent_package(&self, participant: usize) -> Option<Range<usize>> {
        if self.decoded_fragments[participant].is_empty() {
            return None;
        }
        let mut range = None;
        let mut fragment_index = 0;
        loop {
            if let Some(fragment) = self.decoded_fragments[participant].get(fragment_index) {
                let mut size = [0; 4];
                size[..AP_LENGTH_INDEX_SIZE_BYTES]
                    .copy_from_slice(&fragment[..AP_LENGTH_INDEX_SIZE_BYTES]);
                let size = u32::from_le_bytes(size);
                let length =
                    (size as usize + AP_LENGTH_INDEX_SIZE_BYTES).div_ceil(FRAGMENT_SIZE_BYTES);
                let end = fragment_index + length;
                range = Some(Range {
                    start: fragment_index,
                    end,
                });
                fragment_index = end;
            } else {
                break;
            }
        }
        range
    }
}

#[cfg(test)]
mod tests {
    use crate::data_structures::{Frame, Package, WideFactor};
    use crate::matrix::Matrix;
    use crate::network_coding::{Epoch, Equation, GaloisField2p4};
    use crate::{FRAGMENTS_PER_PARTICIPANT_PER_EPOCH, FRAME_SIZE_BYTES};
    use image::{ImageBuffer, Luma};
    use qrcode::QrCode;
    use rand::random;
    use std::array::from_fn;
    use std::fs::File;
    use std::io::Write;

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
    fn matrix_elimination_test_0() {
        let file_0 = File::open("../../media/qr_test/data_0.txt").unwrap();
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
        let mut matrix = Matrix::default();
        for _ in 0..equations.len() {
            let eq = equations
                .iter()
                .cloned()
                .fold(Equation::default(), |acc, e| {
                    acc + (e * (random::<u8>() & 0xF))
                });
            matrix.inner.push(eq);
        }
        matrix.matrix_elimination();
        for (idx, eq) in matrix.inner.iter().enumerate() {
            assert!(eq.factors.is_plain());
            assert_eq!(eq.fragment, fragments[idx]);
        }
    }
    #[test]
    fn push_frame_test_0() {
        let mut e = Epoch::default();
        let package: Package =
            Package::from_read(File::open("../../media/qr_test/data_0.txt").unwrap());
        e.write(package);
        assert!(e.get_package(0, 0).is_some());
        let mut frames = Vec::new();
        for _ in 0..4 {
            let frame = e.pop_recent_frame();
            assert!(frame.is_some());
            frames.push(frame.unwrap());
        }
        let mut e = Epoch::default();
        e.header.participant = 1;
        for (idx, frame) in frames.iter().enumerate() {
            if idx > 0 {
                assert!(!e.equations.is_empty());
            } else {
                assert_eq!(e.equations.len(), idx);
            }
            e.push_frame(frame.clone());
        }
        assert!(e.equations.is_empty());
    }
    #[test]
    fn push_frame_test_1() {
        let mut e_out = Epoch::default();
        assert_eq!(FILES.len(), 4);
        for (idx, file_name) in FILES.iter().enumerate() {
            let ap = Package::from_read(
                File::open(format!("../../media/qr_test/{}", file_name)).unwrap(),
            );
            e_out.write(ap);
            e_out.header.participant += 1;
            assert!(e_out.get_package(idx, 0).is_some());
        }
        let mut e_in = Epoch::default();
        e_in.header.participant = FILES.len() as u8;
        let width = e_out
            .pop_recent_frame()
            .unwrap()
            .factors
            .widths
            .iter()
            .fold(0u16, |acc, w| acc + (*w as u16 * 2));
        let times = width + 20;
        for idx in 0..times {
            let frame = e_out.pop_recent_frame();
            assert!(frame.is_some());
            assert_eq!(e_in.equations.len(), idx as usize);
            e_in.push_frame(frame.unwrap());
            if idx > 0 && e_in.equations.is_empty() {
                println!("decoded after the {idx}. out of {times} frames 🥳");
                break;
            }
        }
        assert!(e_in.equations.is_empty());
    }
    #[test]
    fn push_frame_test_2() {
        let mut e_out = Epoch::default();
        assert_eq!(FILES.len(), 4);
        for (idx, file_name) in FILES.iter().enumerate() {
            let ap = Package::from_read(
                File::open(format!("../../media/qr_test/{}", file_name)).unwrap(),
            );
            e_out.write(ap);
            e_out.header.participant += 1;
            assert!(e_out.get_package(idx, 0).is_some());
        }
        let mut e_in = Epoch::default();
        e_in.header.participant = FILES.len() as u8;
        let mut idx: usize = 0;
        loop {
            let frame = e_out.pop_recent_frame();
            assert!(frame.is_some());
            if e_in.equations.len() != idx || idx > 300 {
                println!("decoded after the {idx}. frame 🥳");
                break;
            }
            if idx >= 196 && idx % 25 == 0 {
                println!("Working on frame {idx}");
            }
            e_in.push_frame(frame.unwrap());
            idx += 1;
        }
        for (idx, file_name) in FILES.iter().enumerate() {
            let mut ap = e_in.get_package(idx, 0).unwrap();
            let mut file = File::create(format!("tests/out_dir/{}", file_name)).unwrap();
            file.write_all(ap.data.as_mut_slice()).unwrap();
        }
    }
    const NUM_FRAMES: usize = 220;
    // const FILES: [&str; 1] = ["data_0.txt"];
    const FILES: [&str; 2] = ["data_0.txt", "data_1.txt"];
    // const FILES: [&str; 4] = [
    //     "data_0.txt",
    //     "data_1.txt",
    //     "dataset-card.png",
    //     "homepage.md",
    // ];
    #[test]
    #[ignore]
    fn generate_qr_codes() {
        let mut e = Epoch::default();
        for (idx, file_name) in FILES.iter().enumerate() {
            let file = File::open(format!("../../media/qr_test/{}", file_name)).unwrap();
            let package = Package::from_read(&file);
            e.header.participant = idx as u8;
            e.write(package);
        }
        for idx in 0..NUM_FRAMES {
            let frame: Frame = e.pop_recent_frame().unwrap();
            let code: Result<QrCode, _> = frame.try_into();
            if let Ok(qr) = code {
                let image: ImageBuffer<Luma<u8>, Vec<u8>> = qr.render::<Luma<u8>>().build();
                image.save(format!("tests/out_dir/qr_{idx}.png")).unwrap();
            }
        }
    }
    #[test]
    #[ignore]
    fn scan_qr_codes() {
        let mut e = Epoch::default();
        for (idx, code) in (0..NUM_FRAMES)
            .filter_map(|file_idx| image::open(format!("tests/out_dir/qr_{}.png", file_idx)).ok())
            .enumerate()
        {
            let img = code.to_luma8();
            let mut img = rqrr::PreparedImage::prepare(img);
            let grids = img.detect_grids();
            let mut buf = Vec::new();
            if let Some(grid) = grids.get(0)
                && let Ok(_) = grid.decode_to(&mut buf)
            {
                let data: [u8; FRAME_SIZE_BYTES] = from_fn(|idx| *buf.get(idx).unwrap_or(&0));
                let frame: Frame = data.into();
                e.push_frame(frame);
                if e.equations.is_empty() {
                    println!("Decoded after the {idx}. QR-Code");
                    break;
                }
            }
        }
        for (participant_idx, file) in FILES.iter().enumerate() {
            let maybe_ap = e.get_package(participant_idx, 0);
            assert!(maybe_ap.is_some());
            let Package {
                mut data,
                size: _size,
            } = maybe_ap.unwrap();
            if let Ok(mut file) = File::create(format!("tests/out_dir/{}", file)) {
                let _ = file.write_all(&mut data);
            }
        }
    }
    #[test]
    fn coding_test_0() {
        let mut e_out = Epoch::default();
        for (idx, file_name) in FILES.iter().enumerate() {
            let ap = Package::from_read(
                File::open(format!("../../media/qr_test/{}", file_name)).unwrap(),
            );
            e_out.write(ap);
            e_out.header.participant += 1;
            assert!(e_out.get_package(idx, 0).is_some());
        }
        let mut e_in = Epoch::default();
        e_in.header.participant = FILES.len() as u8;
        let mut idx: usize = 0;
        loop {
            let frame = e_out.pop_recent_frame().unwrap();
            if let Ok(code) = frame.try_into() {
                let image: ImageBuffer<Luma<u8>, Vec<u8>> =
                    QrCode::render::<Luma<u8>>(&code).build();
                image.save(format!("tests/out_dir/qr_{idx}.png")).unwrap();
                let file = image::open(format!("tests/out_dir/qr_{}.png", idx)).unwrap();
                let img = file.to_luma8();
                let mut img = rqrr::PreparedImage::prepare(img);
                let grids = img.detect_grids();
                let mut buf = Vec::new();
                if let Some(grid) = grids.get(0)
                    && let Ok(_) = grid.decode_to(&mut buf)
                {
                    println!("Working on {idx}. frame");
                    let data: [u8; FRAME_SIZE_BYTES] = from_fn(|idx| *buf.get(idx).unwrap_or(&0));
                    let frame: Frame = data.into();
                    e_in.push_frame(frame);
                    idx += 1;
                    if (0..FILES.len())
                        .into_iter()
                        .all(|i| !e_in.decoded_fragments[i].is_empty())
                    {
                        println!("decoded after {idx} frames 🥳");
                        break;
                    }
                }
            }
        }
        for (idx, file_name) in FILES.iter().enumerate() {
            let mut ap = e_in.get_package(idx, 0).unwrap();
            let mut file = File::create(format!("tests/out_dir/{}", file_name)).unwrap();
            file.write_all(ap.data.as_mut_slice()).unwrap();
        }
    }
}
