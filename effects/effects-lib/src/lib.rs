//! Shared helpers for xclock effects.
//!
//! Effects are executables that respond with animation frames. This crate
//! re-exports the plugin protocol types and provides common frame builders.

pub use xclock_plugin_api::{
    EffectTarget, Frame, Request, Response, read_request, serde_json, write_response,
};

use serde_json::Value;

const GLYPHS: &[char] = &['#', '@', '%', '&', '*', '+', '=', '-', '.', ':', '?', '!'];

/// Builds a progress bar line of exactly `width` characters.
pub fn bar_line(width: usize, progress: f64, fill: char, empty: char) -> String {
    let width = width.max(1);
    let filled = (progress.clamp(0.0, 1.0) * width as f64).round() as usize;

    let mut text = String::with_capacity(width);
    for position in 0..width {
        text.push(if position < filled { fill } else { empty });
    }
    text
}

/// Progress reported by the kernel for `bar` effects.
pub fn request_progress(request: &Request) -> f64 {
    request.context.progress.unwrap_or(0.0).clamp(0.0, 1.0)
}

/// Width requested by the kernel for the effect output.
pub fn request_width(request: &Request, default: usize) -> usize {
    match request.context.width {
        0 => default,
        width => width as usize,
    }
}

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

    #[test]
    fn bar_line_fills_proportionally() {
        assert_eq!(bar_line(8, 0.5, '#', '-'), "####----");
        assert_eq!(bar_line(4, 0.0, '#', '-'), "----");
        assert_eq!(bar_line(4, 1.5, '#', '-'), "####");
    }

    #[test]
    fn request_helpers_read_context() {
        let request: Request = serde_json::from_str(
            r#"{
                "version": 1,
                "kind": "effect",
                "target": "bar",
                "args": {},
                "context": { "now": "now", "width": 12, "height": 1, "progress": 0.25 }
            }"#,
        )
        .unwrap();

        assert_eq!(request.target, EffectTarget::Bar);
        assert_eq!(request_progress(&request), 0.25);
        assert_eq!(request_width(&request, 8), 12);

        let fallback: Request = serde_json::from_str(
            r#"{
                "version": 1,
                "kind": "effect",
                "args": {},
                "context": { "now": "now", "width": 0, "height": 1 }
            }"#,
        )
        .unwrap();
        assert_eq!(request_width(&fallback, 8), 8);
    }
}
