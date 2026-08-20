#[derive(Copy, Clone, Debug, Default)]
pub struct FrameHeader {
    pub participant: u8,
    pub is_overflowing: bool,
    pub epoch: u8,
}
