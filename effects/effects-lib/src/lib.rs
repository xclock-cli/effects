//! Shared helpers for xclock effects.
//!
//! Effects are executables that respond with animation frames. This crate
//! re-exports the plugin protocol types and provides common frame builders.

pub use xclock_plugin_api::{Frame, Response, read_request, serde_json, write_response};

use serde_json::Value;

const GLYPHS: &[char] = &['#', '@', '%', '&', '*', '+', '=', '-', '.', ':', '?', '!'];

/// Reads a list of lines from effect args, falling back to a demo line.
pub fn lines_from_args(args: &Value) -> Vec<String> {
    args.get("lines")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .filter(|lines| !lines.is_empty())
        .unwrap_or_else(|| vec!["xclock".to_string()])
}

pub fn delay_from_args(args: &Value, default: u64) -> u64 {
    args.get("delay_ms")
        .and_then(Value::as_u64)
        .unwrap_or(default)
}

/// Reveals text character by character, padding unrevealed cells with spaces.
pub fn typewriter_frames(lines: &[String], delay_ms: u64) -> Vec<Frame> {
    let total = total_chars(lines);
    let mut frames = Vec::with_capacity(total);

    for step in 1..=total {
        frames.push(Frame {
            delay_ms,
            lines: reveal(lines, step, |_| ' '),
        });
    }

    frames
}

/// Reveals text through random glyphs until each character resolves.
pub fn decrypt_frames(lines: &[String], delay_ms: u64, seed: u64) -> Vec<Frame> {
    let total = total_chars(lines);
    let mut frames = Vec::with_capacity(total + 1);

    for step in 0..=total {
        frames.push(Frame {
            delay_ms,
            lines: reveal(lines, step, |character| {
                if character == ' ' {
                    ' '
                } else {
                    let index = (seed.wrapping_add(step as u64 * 31)) as usize;
                    GLYPHS[index % GLYPHS.len()]
                }
            }),
        });
    }

    frames
}

fn total_chars(lines: &[String]) -> usize {
    lines.iter().map(|line| line.chars().count()).sum()
}

fn reveal<F>(lines: &[String], visible: usize, scramble: F) -> Vec<String>
where
    F: Fn(char) -> char,
{
    let mut remaining = visible;
    let mut result = Vec::with_capacity(lines.len());

    for line in lines {
        let characters: Vec<char> = line.chars().collect();
        let mut text = String::with_capacity(characters.len());
        for (index, character) in characters.iter().enumerate() {
            if index < remaining {
                text.push(*character);
            } else {
                text.push(scramble(*character));
            }
        }
        remaining = remaining.saturating_sub(characters.len());
        result.push(text);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines() -> Vec<String> {
        vec!["ab".to_string(), "cd".to_string()]
    }

    #[test]
    fn typewriter_reveals_progressively() {
        let frames = typewriter_frames(&lines(), 50);

        assert_eq!(frames.len(), 4);
        assert_eq!(frames[0].lines, vec!["a ".to_string(), "  ".to_string()]);
        assert_eq!(frames[3].lines, lines());
        assert_eq!(frames[0].delay_ms, 50);
    }

    #[test]
    fn decrypt_starts_scrambled_and_ends_clean() {
        let frames = decrypt_frames(&lines(), 40, 7);

        assert_eq!(frames.len(), 5);
        assert_eq!(frames.last().unwrap().lines, lines());
        assert_ne!(frames[0].lines, lines());
        assert!(frames[0].lines[0].chars().count() == 2);
    }

    #[test]
    fn args_helpers_fall_back() {
        assert_eq!(lines_from_args(&Value::Null), vec!["xclock".to_string()]);
        assert_eq!(delay_from_args(&Value::Null, 80), 80);
        assert_eq!(
            lines_from_args(&serde_json::json!({ "lines": ["a"] })),
            vec!["a".to_string()]
        );
        assert_eq!(
            delay_from_args(&serde_json::json!({ "delay_ms": 10 }), 80),
            10
        );
    }
}
