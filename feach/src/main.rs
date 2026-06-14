use feach::run;
use togail::Game;

fn main() {
    let mut game = Game::default();
    run(|inputs, delta_ms| {
        game.tick(inputs, delta_ms);
        game.get_frame()
    });
}
