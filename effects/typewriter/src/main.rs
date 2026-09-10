use xclock_effects_lib::{
    Response, delay_from_args, lines_from_args, read_request, typewriter_frames, write_response,
};

fn main() -> std::io::Result<()> {
    let request = read_request()?;
    let lines = lines_from_args(&request.args);
    let delay_ms = delay_from_args(&request.args, 60);

    write_response(&Response::frames(typewriter_frames(&lines, delay_ms)))
}
