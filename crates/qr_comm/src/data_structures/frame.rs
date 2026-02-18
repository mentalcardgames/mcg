use crate::data_structures::{Fragment, FrameFactor, FrameHeader};
use crate::{FRAME_SIZE_BYTES, QR_CODE_ECC, QR_CODE_VERSION};
use qrcode::types::QrError;
use qrcode::{QrCode, QrResult};

#[derive(Clone)]
pub struct Frame {
    pub factors: FrameFactor,
    pub fragment: Fragment,
    pub header: FrameHeader,
}

impl TryFrom<Frame> for QrCode {
    type Error = QrError;

    fn try_from(value: Frame) -> QrResult<QrCode> {
        QrCode::with_version::<[u8; FRAME_SIZE_BYTES]>(value.into(), QR_CODE_VERSION, QR_CODE_ECC)
    }
}

impl Frame {
    pub fn new(factors: FrameFactor, fragment: Fragment, header: FrameHeader) -> Self {
        Self {
            factors,
            fragment,
            header,
        }
    }
}
