use crate::{
    Frame, GRAVITY_TICK, Input,
    board::{Board, DropOutcome, SpawnOutcome}, random::get_random_shape_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MakeNewShape,
    MergeShape,
    DropShape,
    TakeInput,
    CheckRows,
    GameOver,
}

pub struct Game {
    board: Board,
    game_state: GameState,
    clock: u32,
    shape_state: u64,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::default(),
            game_state: GameState::MakeNewShape,
            clock: 0,
            shape_state: 1,
        }
    }
}

impl Game {
    pub fn new(state: u64) -> Self {
        let mut game = Self::default();
        game.shape_state = state;
        game
    }

    fn make_new_shape(&mut self) {
        let shape_type = get_random_shape_type(&mut self.shape_state);
        self.game_state = match self.board.add_new_shape(shape_type) {
            SpawnOutcome::Spawned => GameState::TakeInput,
            SpawnOutcome::FullBoard => GameState::GameOver,
        };
    }

    fn drop_shape(&mut self) {
        self.clock = 0;
        self.game_state = match self.board.drop_shape() {
            DropOutcome::Landed => GameState::MergeShape,
            DropOutcome::Dropped => GameState::TakeInput,
        }
    }

    fn merge_shape(&mut self) {
        self.board.merge_shape();
        self.game_state = GameState::CheckRows;
    }

    fn check_rows(&mut self) {
        self.board.check_rows();
        self.game_state = GameState::MakeNewShape;
    }

    fn take_input(&mut self, input: Option<Input>) {
        if self.clock > GRAVITY_TICK {
            self.game_state = GameState::DropShape;
            return;
        }
        let Some(input) = input else { return };
        match input {
            Input::Left | Input::Right | Input::SoftDrop => self.board.move_shape(input),
            Input::RotateCw | Input::RotateCcw => self.board.rotate_shape(input),
            Input::HardDrop => {
                self.board.hard_drop();
                self.game_state = GameState::MergeShape;
            }
            _ => (),
        }
    }

    fn step_state(&mut self, input: Option<Input>) {
        match self.game_state {
            GameState::DropShape => self.drop_shape(),
            GameState::MakeNewShape => self.make_new_shape(),
            GameState::CheckRows => self.check_rows(),
            GameState::TakeInput => self.take_input(input),
            GameState::MergeShape => self.merge_shape(),
            GameState::GameOver => (),
        }
    }

    pub fn tick(&mut self, inputs: &[Input], delta_ms: u32) {
        // HACK: Handle multiple inputs.
        let input = inputs.first().copied();
        self.clock += delta_ms;
        self.step_state(input);
    }

    pub fn get_frame(&self) -> Frame {
        let buffer = self.board.render_cells();
        Frame {
            board: buffer,
            score: 0,
            level: 1,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn gravity_does_not_fire_after_shape_spawn() {
        let mut game = Game::default();
        let inputs = [];
        game.tick(&inputs, 1); // spawns shape, enters TakeInput
        game.tick(&inputs, 1); // should stay in TakeInput
        assert_eq!(game.game_state, GameState::TakeInput);
    }

    #[test]
    fn new_game_starts_in_make_new_shape_state() {
        let game = Game::default();
        assert_eq!(game.game_state, GameState::MakeNewShape);
    }

    #[test]
    fn first_tick_spawns_a_shape() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert!(game.board.get_shape().is_some());
    }

    #[test]
    fn clock_accumulates_delta_ms_each_tick() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 110);
        assert_eq!(game.clock, 110);
    }

    #[test]
    fn take_input_with_clock_over_threshold_moves_to_drop_shape() {
        let mut game = Game::default();
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.game_state, GameState::DropShape);
    }

    #[test]
    fn take_input_with_clock_under_threshold_does_not_transition() {
        let mut game = Game::default();
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK - 10);
        assert_eq!(game.game_state, GameState::TakeInput);
    }

    #[test]
    fn shape_drops_with_drop_shape() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        let y_1 = game.board.get_shape_pos().clone().unwrap().y;
        game.drop_shape();
        let y_2 = game.board.get_shape_pos().clone().unwrap().y;
        assert!(y_1 < y_2)
    }

    #[test]
    fn shape_drops_when_clock_over_threshold() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        let y_1 = game.board.get_shape_pos().clone().unwrap().y;
        game.tick(&inputs, GRAVITY_TICK + 10);
        game.tick(&inputs, 16);
        let y_2 = game.board.get_shape_pos().clone().unwrap().y;
        assert!(y_1 < y_2)
    }

    #[test]
    fn clock_resets_after_drop_shape() {
        let mut game = Game::default();
        game.tick(&[], 16);
        game.clock = GRAVITY_TICK + 10;
        game.drop_shape();
        assert_eq!(game.clock, 0);
    }

    #[test]
    fn make_new_shape_state_moves_to_take_input() {
        let mut game = Game::default();
        game.game_state = GameState::MakeNewShape;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.game_state, GameState::TakeInput);
    }

    #[test]
    fn take_input_transitions_to_take_input() {
        let mut game = Game::default();
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.game_state, GameState::TakeInput);
    }

    #[test]
    fn merge_shape_transitions_to_check_rows() {
        let mut game = Game::default();
        game.game_state = GameState::MergeShape;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.game_state, GameState::CheckRows);
    }

    #[test]
    fn check_rows_transitions_to_make_new_shape() {
        let mut game = Game::default();
        game.game_state = GameState::CheckRows;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.game_state, GameState::MakeNewShape);
    }

    #[test]
    fn hard_drop_input_immediately_transitions_to_merge_shape() {
        let mut game = Game::default();
        game.tick(&[], 1); // spawn shape → TakeInput
        game.tick(&[Input::HardDrop], 1);
        assert_eq!(game.game_state, GameState::MergeShape);
    }

    #[test]
    fn game_over_stays_in_game_over() {
        let mut game = Game::default();
        game.game_state = GameState::GameOver;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.game_state, GameState::GameOver)
    }
}
