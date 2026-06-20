use alloc::vec;
use alloc::vec::Vec;

use crate::{
    Frame, GRAVITY_TICK, Input,
    board::{Board, DropOutcome, SpawnOutcome},
    random::get_random_shape_type,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MakeNewShape,
    MergeShape,
    DropShape,
    TakeInput,
    CheckRows,
    GameOver,
    Pause,
}

pub struct Game {
    board: Board,
    game_state: GameState,
    clock: u32,
    shape_state: u64,
    input_buffer: Vec<Input>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::default(),
            game_state: GameState::MakeNewShape,
            clock: 0,
            shape_state: 1,
            input_buffer: vec![],
        }
    }
}

impl Game {
    pub fn new(state: u64) -> Self {
        Game {
            shape_state: state,
            ..Default::default()
        }
    }

    fn make_new_shape(&mut self) {
        let shape_type = get_random_shape_type(&mut self.shape_state);
        self.game_state = match self.board.add_new_shape(shape_type) {
            SpawnOutcome::Spawned => GameState::TakeInput,
            SpawnOutcome::FullBoard => GameState::GameOver,
        };
        self.input_buffer.retain(|it| matches!(it, Input::Pause));
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

    fn take_input(&mut self) {
        if self.clock > GRAVITY_TICK {
            self.game_state = GameState::DropShape;
            return;
        }
        for input in self.input_buffer.drain(..) {
            match input {
                Input::Pause => self.game_state = GameState::Pause,
                Input::Left | Input::Right | Input::SoftDrop => self.board.move_shape(input),
                Input::RotateCw | Input::RotateCcw => self.board.rotate_shape(input),
                Input::HardDrop => {
                    self.board.hard_drop();
                    self.game_state = GameState::MergeShape;
                }
                _ => (),
            }
        }
    }

    fn pause(&mut self) {
        for input in self.input_buffer.drain(..) {
            self.game_state = match input {
                Input::Pause => GameState::TakeInput,
                _ => GameState::Pause,
            }
        }
    }

    fn game_over(&mut self) {
        self.input_buffer = vec![];
    }

    fn step_state(&mut self) {
        match self.game_state {
            GameState::DropShape => self.drop_shape(),
            GameState::MakeNewShape => self.make_new_shape(),
            GameState::CheckRows => self.check_rows(),
            GameState::TakeInput => self.take_input(),
            GameState::MergeShape => self.merge_shape(),
            GameState::Pause => self.pause(),
            GameState::GameOver => self.game_over(),
        }
    }

    pub fn tick(&mut self, inputs: &[Input], delta_ms: u32) {
        self.input_buffer.extend_from_slice(inputs);
        self.clock += delta_ms;
        self.step_state();
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

    #[test]
    fn game_in_pause_stays_in_pause() {
        let mut game = Game::default();
        game.game_state = GameState::Pause;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        assert_eq!(game.game_state, GameState::Pause)
    }

    #[test]
    fn game_in_pause_goes_to_take_input_on_pause() {
        let mut game = Game::default();
        game.game_state = GameState::Pause;
        let inputs: [Input; 1] = [Input::Pause];
        game.tick(&inputs, 16);
        assert_eq!(game.game_state, GameState::TakeInput)
    }

    #[test]
    fn input_in_non_input_frame_applies_input_at_next_input_frame() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        game.game_state = GameState::DropShape;
        // Pass input into game and move to TakeInput state
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 16);
        let x_0 = game.board.get_shape_pos().unwrap().x;
        // Iter through to next TakeInput state.
        while game.game_state != GameState::TakeInput {
            let inputs: [Input; 0] = [];
            game.tick(&inputs, 16);
        }
        // Pass no input on TakeInput state.
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn input_in_non_input_frame_applies_multiple_input_at_next_input_frame() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        game.game_state = GameState::DropShape;
        // Pass input into game and move to TakeInput state
        let inputs: [Input; 2] = [Input::Left, Input::Left];
        game.tick(&inputs, 16);
        let x_0 = game.board.get_shape_pos().unwrap().x;
        // Iter through to next TakeInput state.
        while game.game_state != GameState::TakeInput {
            let inputs: [Input; 0] = [];
            game.tick(&inputs, 16);
        }
        // Pass no input on TakeInput state.
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 2);
    }

    #[test]
    fn move_on_gravity_tick_drops_gravity() {
        let mut game = Game::default();
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.game_state, GameState::DropShape)
    }

    #[test]
    fn move_on_gravity_tick_does_not_move_shape() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        let x_0 = game.board.get_shape_pos().unwrap().x;
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, GRAVITY_TICK + 1);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0);
    }

    #[test]
    fn move_on_gravity_tick_moves_shape_after_next_frame() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        let x_0 = game.board.get_shape_pos().unwrap().x;
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, GRAVITY_TICK + 1);
        // Game state now DropShape
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        // Game state now TakeInput
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn move_on_take_input_moves_shape() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        let x_0 = game.board.get_shape_pos().unwrap().x;
        game.game_state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 1);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn non_pause_from_before_make_shape_is_dropped() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.input_buffer = vec![
            Input::Left,
            Input::Left,
            Input::Right,
            Input::Pause,
            Input::SoftDrop,
        ];
        game.game_state = GameState::MakeNewShape;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.input_buffer, vec![Input::Pause]);
    }

    #[test]
    fn game_over_drains_input() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.input_buffer = vec![
            Input::Left,
            Input::Left,
            Input::Right,
            Input::Pause,
            Input::SoftDrop,
        ];
        game.game_state = GameState::GameOver;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.input_buffer, vec![]);
    }
}
