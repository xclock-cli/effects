use xclock_effects_lib::{
    Frame, Response, bar_line, delay_from_args, read_request, request_progress, request_width,
    serde_json, write_response,
};

fn main() -> std::io::Result<()> {
    let request = read_request()?;
    let width = request_width(&request, 24);
    let progress = request_progress(&request);
    let delay = delay_from_args(&request.args, 120);

    let fill = character(&request.args, "fill", '█');
    let empty = character(&request.args, "empty", '░');
    let shimmer = character(&request.args, "shimmer", '▓');

    let filled = (progress * width as f64).round() as usize;
    let mut frames = Vec::with_capacity(8);

    for step in 0..8 {
        let mut line = bar_line(width, progress, fill, empty);
        if filled > 0 {
            let position = step % filled;
            let mut cells: Vec<char> = line.chars().collect();
            if position < cells.len() {
                cells[position] = shimmer;
            }
            line = cells.into_iter().collect();
        }
        frames.push(Frame {
            delay_ms: delay,
            lines: vec![line],
        });
    }

    write_response(&Response::frames(frames))
}

fn character(args: &serde_json::Value, key: &str, default: char) -> char {
    args.get(key)
        .and_then(|value| value.as_str())
        .and_then(|value| value.chars().next())
        .unwrap_or(default)
}
