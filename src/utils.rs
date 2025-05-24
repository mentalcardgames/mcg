use sha2::{Sha256, Digest};
use std::char;

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

/// Hashes arbitrary data into a sequence of three emojis.
///
/// This function uses SHA-256 to hash the input data and then maps the hash
/// to three emojis from the Unicode emoji ranges.
///
/// # Arguments
///
/// * `data` - A byte slice containing the data to hash
///
/// # Returns
///
/// A `String` containing three emojis
///
/// # Examples
///
/// ```
/// let hash = emoji_hash("Hello, world!".as_bytes());
/// println!("Emoji hash: {}", hash);
/// ```
pub fn emoji_hash(data: &[u8]) -> String {
    // Define emoji Unicode ranges
    // These ranges cover the most common emoji codepoints
    const EMOJI_RANGES: &[(u32, u32)] = &[
        // Emoticons
        (0x1F600, 0x1F64F),
        // Miscellaneous Symbols and Pictographs
        (0x1F300, 0x1F5FF),
        // Supplemental Symbols and Pictographs
        (0x1F900, 0x1F9FF),
        // Transport and Map Symbols
        (0x1F680, 0x1F6FF),
        // Additional Emoticons
        (0x1F910, 0x1F96B),
        // Symbols and Pictographs Extended-A
        (0x1FA70, 0x1FAFF),
        // Additional symbols
        (0x2600, 0x26FF),
        // Dingbats
        (0x2700, 0x27BF),
    ];
    
    // Calculate total number of emojis in all ranges
    let total_emojis: u32 = EMOJI_RANGES.iter()
        .map(|(start, end)| end - start + 1)
        .sum();
    
    // Calculate the SHA-256 hash of the input data
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut emoji_string = String::new();
    
    for i in 0..3 {
        // Extract 4 bytes from the hash (to have more entropy)
        let idx_bytes = [result[i*4], result[i*4 + 1], result[i*4 + 2], result[i*4 + 3]];
        // Convert to u32 and mod by the total number of emojis
        let mut emoji_idx = u32::from_be_bytes(idx_bytes) % total_emojis;
        
        // Find which range this index falls into
        let mut code_point = 0;
        for (start, end) in EMOJI_RANGES {
            let range_size = end - start + 1;
            if emoji_idx < range_size {
                code_point = start + emoji_idx;
                break;
            }
            emoji_idx -= range_size;
        }
        
        // Convert the code point to a character and add to the string
        if let Some(emoji_char) = char::from_u32(code_point) {
            emoji_string.push(emoji_char);
        }
    }
    
    emoji_string
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emoji_hash() {
        let hash1 = emoji_hash("Hello, world!".as_bytes());
        let hash2 = emoji_hash("Hello, world!".as_bytes());
        let hash3 = emoji_hash("Different input".as_bytes());
        
        // Same input should produce same output
        assert_eq!(hash1, hash2);
        
        // Different input should produce different output
        assert_ne!(hash1, hash3);
        
        // Output should be 3 emoji characters
        assert_eq!(hash1.chars().count(), 3);
    }
}