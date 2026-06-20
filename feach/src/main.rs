use feach::{parse_debug_level, run};
use std::time::{SystemTime, UNIX_EPOCH};
use togail::Game;

fn main() {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);
    let args: Vec<String> = std::env::args().collect();
    let debug_level = parse_debug_level(&args);
    let mut game = Game::new_at_level(seed, debug_level);
    run(|inputs, delta_ms| {
        game.tick(inputs, delta_ms);
        game.get_frame()
    });
}
