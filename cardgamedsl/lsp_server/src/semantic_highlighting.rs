use crate::error_to_diagnostics::to_range;
use front_end::{
    ast::ast_spanned::SGame,
    symbols::{GameType, SymbolVisitor},
    walker::Walker,
};

pub struct AbsoluteToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

/// Converts a list of absolute semantic tokens into a relative integer array (deltas).
///
/// This implements the LSP [Semantic Tokens] specification, transforming absolute 
/// positions into a flattened `Vec<u32>` where each token is represented by 5 values:
/// 1. `deltaLine`: Line relative to the previous token.
/// 2. `deltaStart`: Start character relative to the previous token (if on the same line) 
///    or absolute start character (if on a new line).
/// 3. `length`: The length of the token.
/// 4. `tokenType`: The integer index of the token type.
/// 5. `tokenModifiers`: A bitmask of modifiers.
///
/// ### Important
/// Tokens **must** be sorted by line and then by start character for the delta 
/// calculation to be valid. This function performs that sort internally.
///
/// [Semantic Tokens]: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.16/specification/#textDocument_semanticTokens
pub fn calculate_deltas(mut tokens: Vec<AbsoluteToken>) -> Vec<u32> {
    // Sort tokens by line, then by start character (crucial for deltas!)
    tokens.sort_by_key(|t| (t.line, t.start));

    let mut last_line = 0;
    let mut last_start = 0;
    let mut output = Vec::with_capacity(tokens.len() * 5);

    for token in tokens {
        let delta_line = token.line - last_line;
        let delta_start = if delta_line == 0 {
            token.start - last_start
        } else {
            token.start
        };

        output.push(delta_line);
        output.push(delta_start);
        output.push(token.length);
        output.push(token.token_type);
        output.push(token.modifiers);

        last_line = token.line;
        last_start = token.start;
    }
    output
}

/// Transforms an AST (`SGame`) into a flat list of [`AbsoluteToken`]s for semantic highlighting.
///
/// ### Process
/// 1. **Symbol Collection:** Uses a `SymbolVisitor` to walk the AST and collect all 
///    relevant identifiers and symbols.
/// 2. **Name Resolution:** Resolves the collected symbols to determine their 
///    specific types (e.g., Variable, Function, Constant).
/// 3. **LSP Mapping:** Maps the resolved symbols and their source spans into 
///    [`AbsoluteToken`] structures, translating spans into 0-indexed line and character offsets.
///
/// ### Token Length Handling
/// This function calculates the token length based on its span. If a token appears 
/// to span multiple lines, it currently uses the raw span difference as a fallback, 
/// though most identifiers are expected to be single-line.
///
/// ### Future Enhancements
/// * **Modifiers:** Currently defaults to `0`. Can be updated to include bitmasks 
///   for `readonly`, `static`, or `deprecated` status based on the `g_type`.
pub fn tokenize_ast(ast: &SGame) -> Vec<AbsoluteToken> {
    let mut symbols = SymbolVisitor::new();
    ast.walk(&mut symbols);

    let var_type = symbols.name_resolution();

    var_type
        .iter()
        .map(|(v, g_type)| {
            let range = to_range(&v.span);

            AbsoluteToken {
                line: range.start.line,
                start: range.start.character,
                // Calculate length: handle multi-line spans by focusing on the start line length
                length: if range.end.line == range.start.line {
                    range.end.character - range.start.character
                } else {
                    // If a token spans multiple lines, LSP usually expects
                    // a separate entry per line, but for identifiers, this is rare.
                    (v.span.end - v.span.start) as u32
                },
                token_type: game_type_to_legend_index(g_type),
                modifiers: 0, // You can add "readonly" or "static" logic here later
            }
        })
        .collect()
}

fn game_type_to_legend_index(gt: &GameType) -> u32 {
    match gt {
        GameType::Player => 0,
        GameType::Team => 1,
        GameType::Location => 2,
        GameType::Precedence => 3,
        GameType::PointMap => 4,
        GameType::Combo => 5,
        GameType::Key => 6,
        GameType::Value => 7,
        GameType::Memory => 8,
        GameType::Token => 9,
        GameType::Stage => 10,
        GameType::NoType => 11,
    }
}
