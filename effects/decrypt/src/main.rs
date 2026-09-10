use xclock_effects_lib::{
    Response, decrypt_frames, delay_from_args, lines_from_args, read_request, write_response,
};

fn main() -> std::io::Result<()> {
    let request = read_request()?;
    let lines = lines_from_args(&request.args);
    let delay_ms = delay_from_args(&request.args, 40);
    let seed = request
        .args
        .get("seed")
        .and_then(|value| value.as_u64())
        .unwrap_or(7);

    write_response(&Response::frames(decrypt_frames(&lines, delay_ms, seed)))
}
