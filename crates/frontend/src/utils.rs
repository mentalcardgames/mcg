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

/// Returns the origin of the current browser location (e.g. "http://127.0.0.1:3000"),
/// or a fallback if not running in a browser.
pub fn get_origin() -> String {
    web_sys::window()
        .and_then(|w| w.location().origin().ok())
        .filter(|o| !o.is_empty() && o != "null")
        .unwrap_or_else(|| "http://127.0.0.1:3000".to_string())
}

/// Returns the host (hostname and optional port, e.g. "127.0.0.1:3000" or "example.com")
/// from the current browser location, if available.
pub fn get_browser_host() -> Option<String> {
    web_sys::window()
        .and_then(|w| w.location().host().ok())
        .filter(|h| !h.is_empty())
}

/// Default server address derived from the browser location,
/// falling back to "127.0.0.1:3000" outside a browser environment or if empty.
pub fn default_server_address() -> String {
    get_browser_host().unwrap_or_else(|| "127.0.0.1:3000".to_string())
}

/// Returns the WebSocket protocol scheme ("wss" for HTTPS pages, "ws" for HTTP / fallback).
pub fn get_websocket_scheme() -> &'static str {
    let is_https = web_sys::window()
        .and_then(|w| w.location().protocol().ok())
        .map(|proto| proto == "https:")
        .unwrap_or(false);

    if is_https {
        "wss"
    } else {
        "ws"
    }
}

/// Format a full WebSocket URL from a server address string.
/// Handles cases where `server_address` is empty, already includes a scheme,
/// or already includes the `/ws` endpoint path.
pub fn format_websocket_url(server_address: &str) -> String {
    let trimmed = server_address.trim();
    let addr = if trimmed.is_empty() {
        default_server_address()
    } else {
        trimmed.to_string()
    };

    if addr.starts_with("ws://") || addr.starts_with("wss://") {
        if addr.contains("/ws") {
            addr
        } else {
            format!("{}/ws", addr.trim_end_matches('/'))
        }
    } else {
        let scheme = get_websocket_scheme();
        let stripped = addr.trim_end_matches('/');
        if stripped.ends_with("/ws") {
            format!("{scheme}://{stripped}")
        } else {
            format!("{scheme}://{stripped}/ws")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_websocket_url() {
        // Without scheme
        assert_eq!(
            format_websocket_url("127.0.0.1:3000"),
            "ws://127.0.0.1:3000/ws"
        );
        assert_eq!(
            format_websocket_url("127.0.0.1:3000/ws"),
            "ws://127.0.0.1:3000/ws"
        );
        assert_eq!(
            format_websocket_url("localhost:8080/"),
            "ws://localhost:8080/ws"
        );

        // With scheme
        assert_eq!(
            format_websocket_url("ws://localhost:3000"),
            "ws://localhost:3000/ws"
        );
        assert_eq!(
            format_websocket_url("wss://example.com/ws"),
            "wss://example.com/ws"
        );
        assert_eq!(
            format_websocket_url("wss://example.com"),
            "wss://example.com/ws"
        );

        // Empty fallback
        assert_eq!(format_websocket_url(""), "ws://127.0.0.1:3000/ws");
        assert_eq!(format_websocket_url("   "), "ws://127.0.0.1:3000/ws");
    }

    #[test]
    fn test_default_server_address_fallback() {
        // In native test environment (no browser window), fallback should be "127.0.0.1:3000"
        assert_eq!(default_server_address(), "127.0.0.1:3000");
    }
}
