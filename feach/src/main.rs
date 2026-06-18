use feach::run;
use std::time::{SystemTime, UNIX_EPOCH};
use togail::Game;

fn main() {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(12345);
    let mut game = Game::new(seed);
    run(|inputs, delta_ms| {
        game.tick(inputs, delta_ms);
        game.get_frame()
    });
}
