use bytes::{Buf, BufMut, BytesMut};
use std::io::{self};

// Simple length-prefixed framing: u32 BE length followed by payload bytes
pub const ALPN_MSG: &str = "mcg/transport/msg/1";

pub fn encode_frame(payload: &[u8]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + payload.len());
    let len = payload.len() as u32;
    buf.put_u32(len);
    buf.extend_from_slice(payload);
    buf
}

pub fn try_parse(mut src: &mut BytesMut) -> io::Result<Option<Vec<u8>>> {
    if src.len() < 4 {
        return Ok(None);
    }
    let len = src.get_u32();
    if src.len() < len as usize {
        // not enough
        // put back len
        let mut restored = BytesMut::with_capacity(4 + src.len());
        restored.put_u32(len);
        restored.extend_from_slice(&src);
        *src = restored;
        return Ok(None);
    }
    let mut out = vec![0u8; len as usize];
    src.copy_to_slice(&mut out);
    Ok(Some(out))
}
