use egui::{Context, FontFamily, FontId};
use sha2::{Digest, Sha256};
use std::char;
use std::collections::HashSet;

#[cfg(feature = "console_error_panic_hook")]
#[allow(dead_code)]
pub fn set_panic_hook() {
    console_error_panic_hook::set_once();
}

pub fn get_available_characters(ctx: &Context) -> HashSet<char> {
    let family = FontFamily::Proportional;
    let font_id = FontId::new(14.0, family);
    ctx.fonts(|f| {
        f.lock()
            .fonts
            .font(&font_id)
            .characters()
            .iter()
            .filter(|(chr, _fonts)| !chr.is_whitespace() && !chr.is_ascii_control())
            .map(|(chr, _)| *chr)
            .collect()
    })
}

pub fn get_available_emojis(ctx: &Context) -> Vec<char> {
    const EMOJI_RANGES: &[(u32, u32)] = &[
        (0x1F600, 0x1F64F),
        (0x2600, 0x26FF),
        (0x1F464, 0x1F49F),
        (0x1F44A, 0x1F450),
        (0x2700, 0x27BF),
        (0x1F300, 0x1F5FF),
    ];
    let available_chars = get_available_characters(ctx);
    let mut emojis = Vec::new();
    for (start, end) in EMOJI_RANGES {
        for code_point in *start..=*end {
            if let Some(emoji_char) = char::from_u32(code_point) {
                if available_chars.contains(&emoji_char) {
                    emojis.push(emoji_char);
                }
            }
        }
    }
    emojis
}

pub fn emoji_hash(data: &[u8], ctx: &Context) -> String {
    let emojis = get_available_emojis(ctx);
    if emojis.is_empty() {
        return "???".to_string();
    }
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let mut emoji_string = String::new();
    for i in 0..3 {
        let idx_bytes = [
            result[i * 4],
            result[i * 4 + 1],
            result[i * 4 + 2],
            result[i * 4 + 3],
        ];
        let emoji_idx = u32::from_be_bytes(idx_bytes) % emojis.len() as u32;
        emoji_string.push(emojis[emoji_idx as usize]);
    }
    emoji_string
}

#[cfg(test)]
mod tests {}
