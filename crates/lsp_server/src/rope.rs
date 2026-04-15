use ropey::Rope;
use tower_lsp::lsp_types::{Position, TextDocumentContentChangeEvent};

#[derive(Debug, Clone)]
pub struct Document {
    pub(crate) rope: Rope,
}

pub fn apply_change(rope: &mut Rope, change: &TextDocumentContentChangeEvent) {
    if let Some(range) = change.range {
        // 1. Convert positions to char indices safely
        let start_char = position_to_char(rope, range.start);
        let end_char = position_to_char(rope, range.end);

        // 2. Clamp indices to the current rope length to prevent panics
        let len = rope.len_chars();
        let safe_start = start_char.min(len);
        let safe_end = end_char.min(len).max(safe_start);

        // 3. Perform the edit
        if safe_start < safe_end {
            rope.remove(safe_start..safe_end);
        }

        if !change.text.is_empty() {
            rope.insert(safe_start, &change.text);
        }
    } else {
        *rope = Rope::from_str(&change.text);
    }
}

/// Converts an LSP [`Position`] (line and UTF-16 column) into a flat character index 
/// within the [`Rope`].
///
/// ### Behavior
/// * **Line Indexing:** If the requested line index is out of bounds, it returns the 
///   total length of the document (the end of the rope).
/// * **UTF-16 to Char Mapping:** Since LSP positions are reported in UTF-16 code units, 
///   this function iterates through the line to correctly handle multi-byte characters 
///   (like emojis) that occupy two UTF-16 units but one character index.
/// * **Newline Handling:** The conversion stops if a newline (`\r` or `\n`) is encountered, 
///   ensuring the offset stays within the logical bounds of the specified line.
///
/// ### Parameters
/// * `rope`: The text buffer to query.
/// * `position`: The LSP position containing the 0-indexed line and character offset.
///
/// ### Returns
/// The absolute `usize` character index suitable for methods like `rope.insert()` or `rope.remove()`.
pub fn position_to_char(rope: &Rope, position: Position) -> usize {
    let line_idx = position.line as usize;
    let utf16_col = position.character as usize;

    // If the line index is beyond the rope, return the very end of the document
    if line_idx >= rope.len_lines() {
        return rope.len_chars();
    }

    let line_start_char = rope.line_to_char(line_idx);
    let line_slice = rope.line(line_idx);

    let mut current_utf16 = 0;
    let mut char_offset = 0;

    for c in line_slice.chars() {
        // If we've reached the target UTF-16 column, stop.
        if current_utf16 >= utf16_col {
            break;
        }

        // Stop if we hit a newline character—LSP positions for a line
        // shouldn't technically include the newline itself.
        if c == '\n' || c == '\r' {
            break;
        }

        current_utf16 += c.len_utf16();
        char_offset += 1;
    }

    line_start_char + char_offset
}
