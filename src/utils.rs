use sha2::{Sha256, Digest};
use std::char;
use std::collections::HashSet;
use egui::{Context, FontId, FontFamily};

#[cfg(feature = "console_error_panic_hook")]
#[allow(dead_code)]
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    console_error_panic_hook::set_once();
}

/// Gets all available characters from the given egui context.
///
/// This function returns a HashSet of characters that are supported by the current egui font.
///
/// # Arguments
///
/// * `ctx` - The egui Context to query for available characters
///
/// # Returns
///
/// A `HashSet<char>` containing all available characters
pub fn get_available_characters(ctx: &Context) -> HashSet<char> {
    // Default font family
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

/// Gets a subset of emoji characters that are available in the egui context.
///
/// This function attempts to find emoji characters from common ranges that are
/// supported by the current egui font.
///
/// # Arguments
///
/// * `ctx` - The egui Context to query for available characters
///
/// # Returns
///
/// A `Vec<char>` containing available emoji characters
pub fn get_available_emojis(ctx: &Context) -> Vec<char> {
    // Define emoji Unicode ranges to check
    const EMOJI_RANGES: &[(u32, u32)] = &[
        // Basic emoticons (smileys, etc.)
        (0x1F600, 0x1F64F),
        // Basic symbols (heart, sun, etc.)
        (0x2600, 0x26FF),
        // Common person emojis
        (0x1F464, 0x1F49F),
        // Hand gestures
        (0x1F44A, 0x1F450),
        // Dingbats
        (0x2700, 0x27BF),
        // Miscellaneous Symbols and Pictographs (basic subset)
        (0x1F300, 0x1F5FF),
    ];
    
    // Get all available characters
    let available_chars = get_available_characters(ctx);
    
    // Filter for emoji characters in our ranges
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

/// Hashes arbitrary data into a sequence of three emojis.
///
/// This function uses SHA-256 to hash the input data and then maps the hash
/// to three emojis that are available in the current egui font.
///
/// # Arguments
///
/// * `data` - A byte slice containing the data to hash
/// * `ctx` - The egui Context to determine available emojis
///
/// # Returns
///
/// A `String` containing three emojis
///
/// # Examples
///
/// ```ignore
/// use mcg::utils::emoji_hash;
/// let hash = emoji_hash("Hello, world!".as_bytes(), &ctx);
/// println!("Emoji hash: {}", hash);
/// ```
pub fn emoji_hash(data: &[u8], ctx: &Context) -> String {
    // Get available emojis from egui context
    let emojis = get_available_emojis(ctx);
    
    // If no emojis are available, return a placeholder
    if emojis.is_empty() {
        return "???".to_string();
    }
    
    // Calculate the SHA-256 hash of the input data
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut emoji_string = String::new();
    
    for i in 0..3 {
        // Extract 4 bytes from the hash (to have more entropy)
        let idx_bytes = [result[i*4], result[i*4 + 1], result[i*4 + 2], result[i*4 + 3]];
        // Convert to u32 and mod by the number of available emojis
        let emoji_idx = u32::from_be_bytes(idx_bytes) % emojis.len() as u32;
        
        // Add the selected emoji to the string
        emoji_string.push(emojis[emoji_idx as usize]);
    }
    
    emoji_string
}



#[cfg(test)]
mod tests {
    use super::*;

    // Note: Tests requiring an egui context are disabled
    // since we can't easily create a context in unit tests
}