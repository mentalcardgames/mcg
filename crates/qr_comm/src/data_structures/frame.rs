use crate::data_structures::{FrameFactor, Fragment, FrameHeader};
use crate::{FRAME_SIZE_BYTES, QR_CODE_ECC, QR_CODE_VERSION};
use qrcode::types::QrError;
use qrcode::{QrCode, QrResult};

#[derive(Copy, Clone)]
pub struct Frame {
    pub coding_factors: FrameFactor,
    pub fragment: Fragment,
    pub header: FrameHeader,
}
impl TryFrom<Frame> for QrCode {
    type Error = QrError;

    fn try_from(value: Frame) -> QrResult<QrCode> {
        QrCode::with_version::<[u8; FRAME_SIZE_BYTES]>(
            value.into(),
            QR_CODE_VERSION,
            QR_CODE_ECC,
        )
    }
}

impl Frame {
    pub fn new(coding_factors: FrameFactor, fragment: Fragment, header: FrameHeader) -> Self {
        Self { coding_factors, fragment, header }
    }
}
