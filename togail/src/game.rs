use alloc::vec::Vec;

use crate::{
    Frame, GRAVITY_TICK, Input,
    board::{Board, DropOutcome, Move, Rotation, SpawnOutcome},
    random::get_random_shape_type,
};

const ROWS_PER_LEVEL: u32 = 10;

fn get_gravity(level: u32) -> u32 {
    (0..level).fold(GRAVITY_TICK, |tick, _| tick * 85 / 100)
}

fn get_score(level: u32, cleared_rows: u32) -> u32 {
    let base_score = match cleared_rows {
        0 => 0,
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => unreachable!(),
    };
    base_score * (level + 1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MakeNewShape,
    MergeShape,
    DropShape,
    TakeInput,
    CheckRows,
    GameOver,
    CheckLevel,
    Pause,
}

pub struct Game {
    board: Board,
    state: GameState,
    clock: u32,
    shape_state: u64,
    input_buffer: Vec<Input>,
    level: u32,
    gravity_tick: u32,
    cleared_rows: u32,
    score: u32,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            board: Board::default(),
            state: GameState::MakeNewShape,
            clock: 0,
            shape_state: 1,
            input_buffer: Vec::new(),
            level: 0,
            gravity_tick: GRAVITY_TICK,
            cleared_rows: 0,
            score: 0,
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

    pub fn new_at_level(shape_state: u64, level: u32) -> Self {
        Game {
            shape_state,
            level,
            cleared_rows: ROWS_PER_LEVEL * level,
            gravity_tick: get_gravity(level),
            ..Default::default()
        }
    }

    fn make_new_shape(&mut self) -> GameState {
        let shape_type = get_random_shape_type(&mut self.shape_state);
        self.input_buffer.retain(|it| matches!(it, Input::Pause));
        match self.board.add_new_shape(shape_type) {
            SpawnOutcome::Spawned => GameState::TakeInput,
            SpawnOutcome::FullBoard => GameState::GameOver,
        }
    }

    fn drop_shape(&mut self) -> GameState {
        self.clock = 0;
        match self.board.drop_shape() {
            DropOutcome::Landed => GameState::MergeShape,
            DropOutcome::Dropped => GameState::TakeInput,
        }
    }

    fn merge_shape(&mut self) -> GameState {
        self.board.merge_shape();
        GameState::CheckRows
    }

    fn check_rows(&mut self) -> GameState {
        let cleared_rows = self.board.check_rows();
        self.score += get_score(self.level, cleared_rows);
        self.cleared_rows += cleared_rows;
        GameState::CheckLevel
    }

    fn check_level(&mut self) -> GameState {
        let new_level = self.cleared_rows / ROWS_PER_LEVEL;
        if new_level != self.level {
            self.level = new_level;
            self.gravity_tick = get_gravity(self.level);
        }
        GameState::MakeNewShape
    }

    fn take_input(&mut self) -> GameState {
        for input in self.input_buffer.drain(..) {
            match input {
                Input::Pause => return GameState::Pause,
                Input::Left => self.board.move_shape(Move::Left),
                Input::Right => self.board.move_shape(Move::Right),
                Input::SoftDrop => self.board.move_shape(Move::Drop),
                Input::RotateCcw => self.board.rotate_shape(Rotation::AntiClockwise),
                Input::RotateCw => self.board.rotate_shape(Rotation::Clockwise),
                Input::HardDrop => {
                    self.board.hard_drop();
                    return GameState::MergeShape;
                }
                Input::Quit => {
                    unreachable!("Quit should not have been passable through to game state engine.")
                }
            };
        }
        if self.clock > self.gravity_tick {
            GameState::DropShape
        } else {
            GameState::TakeInput
        }
    }

    fn pause(&mut self) -> GameState {
        for input in self.input_buffer.drain(..) {
            if input == Input::Pause {
                return GameState::TakeInput;
            }
        }
        GameState::Pause
    }

    fn game_over(&mut self) -> GameState {
        self.input_buffer.clear();
        GameState::GameOver
    }

    fn step_state(&mut self) {
        self.state = match self.state {
            GameState::DropShape => self.drop_shape(),
            GameState::MakeNewShape => self.make_new_shape(),
            GameState::CheckRows => self.check_rows(),
            GameState::TakeInput => self.take_input(),
            GameState::MergeShape => self.merge_shape(),
            GameState::Pause => self.pause(),
            GameState::GameOver => self.game_over(),
            GameState::CheckLevel => self.check_level(),
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
            score: self.score,
            level: self.level,
            ghost: self.board.get_ghost(),
        }
    }
}

#[cfg(test)]
mod tests {

    use alloc::vec;

    use super::*;

    #[test]
    fn gravity_does_not_fire_after_shape_spawn() {
        let mut game = Game::default();
        let inputs = [];
        game.tick(&inputs, 1); // spawns shape, enters TakeInput
        game.tick(&inputs, 1); // should stay in TakeInput
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn new_game_starts_in_make_new_shape_state() {
        let game = Game::default();
        assert_eq!(game.state, GameState::MakeNewShape);
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
        let mut game = Game {
            state: GameState::TakeInput,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.state, GameState::DropShape);
    }

    #[test]
    fn take_input_with_clock_under_threshold_does_not_transition() {
        let mut game = Game {
            state: GameState::TakeInput,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK - 10);
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn shape_drops_with_drop_shape() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        let y_1 = game.board.get_shape_pos().unwrap().y;
        game.drop_shape();
        let y_2 = game.board.get_shape_pos().unwrap().y;
        assert!(y_1 < y_2)
    }

    #[test]
    fn shape_drops_when_clock_over_threshold() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        let y_1 = game.board.get_shape_pos().unwrap().y;
        game.tick(&inputs, GRAVITY_TICK + 10);
        game.tick(&inputs, 16);
        let y_2 = game.board.get_shape_pos().unwrap().y;
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
        let mut game = Game {
            state: GameState::MakeNewShape,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn take_input_transitions_to_take_input() {
        let mut game = Game {
            state: GameState::TakeInput,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn merge_shape_transitions_to_check_rows() {
        let mut game = Game {
            state: GameState::MergeShape,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::CheckRows);
    }

    #[test]
    fn check_rows_transitions_to_check_level() {
        let mut game = Game {
            state: GameState::CheckRows,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::CheckLevel);
    }

    #[test]
    fn check_level_transitions_to_make_new_shape() {
        let mut game = Game {
            state: GameState::CheckLevel,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::MakeNewShape);
    }

    #[test]
    fn level_stays_zero_below_threshold() {
        let mut game = Game {
            cleared_rows: ROWS_PER_LEVEL - 1,
            state: GameState::CheckLevel,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.level, 0);
    }

    #[test]
    fn level_increments_at_threshold() {
        let mut game = Game {
            cleared_rows: ROWS_PER_LEVEL,
            state: GameState::CheckLevel,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.level, 1);
    }

    #[test]
    fn level_jumps_multiple_thresholds_in_one_clear() {
        let mut game = Game {
            cleared_rows: ROWS_PER_LEVEL * 2,
            state: GameState::CheckLevel,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.level, 2);
    }

    #[test]
    fn gravity_tick_updates_when_level_changes() {
        let mut game = Game::default();
        let tick_0 = game.gravity_tick;
        game.cleared_rows = ROWS_PER_LEVEL;
        game.state = GameState::CheckLevel;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert!(game.gravity_tick < tick_0);
    }

    #[test]
    fn gravity_tick_unchanged_when_level_unchanged() {
        let mut game = Game::default();
        let tick_0 = game.gravity_tick;
        game.cleared_rows = ROWS_PER_LEVEL - 1;
        game.state = GameState::CheckLevel;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.gravity_tick, tick_0);
    }

    #[test]
    fn get_gravity_at_level_zero_is_base_gravity_tick() {
        let game = Game::default();
        assert_eq!(get_gravity(game.level), GRAVITY_TICK);
    }

    #[test]
    fn get_gravity_decreases_as_level_increases() {
        let mut game = Game {
            level: 1,
            ..Default::default()
        };
        let tick_1 = get_gravity(game.level);
        game.level = 2;
        let tick_2 = get_gravity(game.level);
        assert!(tick_2 < tick_1);
        assert!(tick_1 < GRAVITY_TICK);
    }

    #[test]
    fn get_frame_reports_current_level() {
        let game = Game {
            level: 3,
            ..Default::default()
        };
        assert_eq!(game.get_frame().level, 3);
    }

    #[test]
    fn get_score_for_no_cleared_rows_is_zero() {
        assert_eq!(get_score(0, 0), 0);
    }

    #[test]
    fn get_score_for_single_at_level_zero() {
        assert_eq!(get_score(0, 1), 100);
    }

    #[test]
    fn get_score_for_double_at_level_zero() {
        assert_eq!(get_score(0, 2), 300);
    }

    #[test]
    fn get_score_for_triple_at_level_zero() {
        assert_eq!(get_score(0, 3), 500);
    }

    #[test]
    fn get_score_for_tetris_at_level_zero() {
        assert_eq!(get_score(0, 4), 800);
    }

    #[test]
    fn get_score_scales_with_level() {
        assert_eq!(get_score(1, 1), 200);
        assert_eq!(get_score(2, 1), 300);
    }

    #[test]
    fn merging_with_no_full_rows_does_not_change_score() {
        let mut game = Game::default();
        game.tick(&[], 1); // spawn shape → TakeInput
        game.state = GameState::CheckRows;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.score, 0);
    }

    #[test]
    fn get_frame_reports_current_score() {
        let game = Game {
            score: 4200,
            ..Default::default()
        };
        assert_eq!(game.get_frame().score, 4200);
    }

    #[test]
    fn hard_drop_input_immediately_transitions_to_merge_shape() {
        let mut game = Game::default();
        game.tick(&[], 1); // spawn shape → TakeInput
        game.tick(&[Input::HardDrop], 1);
        assert_eq!(game.state, GameState::MergeShape);
    }

    #[test]
    fn hard_drop_wins_over_gravity_due_in_the_same_tick() {
        let mut game = Game::default();
        game.tick(&[], 1); // spawn shape → TakeInput
        game.state = GameState::TakeInput;
        let inputs = [Input::HardDrop];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.state, GameState::MergeShape);
    }

    #[test]
    fn game_over_stays_in_game_over() {
        let mut game = Game {
            state: GameState::GameOver,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.state, GameState::GameOver)
    }

    #[test]
    fn game_in_pause_stays_in_pause() {
        let mut game = Game {
            state: GameState::Pause,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 16);
        assert_eq!(game.state, GameState::Pause)
    }

    #[test]
    fn game_in_pause_goes_to_take_input_on_pause() {
        let mut game = Game {
            state: GameState::Pause,
            ..Default::default()
        };
        let inputs: [Input; 1] = [Input::Pause];
        game.tick(&inputs, 16);
        assert_eq!(game.state, GameState::TakeInput)
    }

    #[test]
    fn input_in_non_input_frame_applies_input_at_next_input_frame() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        game.state = GameState::DropShape;
        // Pass input into game and move to TakeInput state
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 16);
        let x_0 = game.board.get_shape_pos().unwrap().x;
        // Iter through to next TakeInput state.
        while game.state != GameState::TakeInput {
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
        game.state = GameState::DropShape;
        // Pass input into game and move to TakeInput state
        let inputs: [Input; 2] = [Input::Left, Input::Left];
        game.tick(&inputs, 16);
        let x_0 = game.board.get_shape_pos().unwrap().x;
        // Iter through to next TakeInput state.
        while game.state != GameState::TakeInput {
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
        game.tick(&[], 1); // spawn shape → TakeInput
        game.state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.state, GameState::DropShape)
    }

    #[test]
    fn move_on_gravity_tick_does_move_shape() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        let x_0 = game.board.get_shape_pos().unwrap().x;
        game.state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, GRAVITY_TICK + 1);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn move_on_gravity_tick_moves_shape_after_next_frame() {
        // Arrange a shape and a non-input state.
        let mut game = Game::default();
        game.make_new_shape();
        let x_0 = game.board.get_shape_pos().unwrap().x;
        game.state = GameState::TakeInput;
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
        game.state = GameState::TakeInput;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 1);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn input_still_applies_at_extremely_high_level_where_gravity_is_near_instant() {
        let mut game = Game::new_at_level(1, 100);
        game.tick(&[], 1); // spawn shape, enters TakeInput
        let x_0 = game.board.get_shape_pos().unwrap().x;
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 16);
        let x_1 = game.board.get_shape_pos().unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn gravity_still_fires_at_extremely_high_level_alongside_input() {
        let mut game = Game::new_at_level(1, 100);
        game.tick(&[], 1); // spawn shape, enters TakeInput
        let inputs: [Input; 1] = [Input::Left];
        game.tick(&inputs, 16);
        assert_eq!(game.state, GameState::DropShape);
    }

    #[test]
    fn non_pause_from_before_make_shape_is_dropped() {
        // Arrange a shape and a non-input state.
        let mut game = Game {
            input_buffer: vec![
                Input::Left,
                Input::Left,
                Input::Right,
                Input::Pause,
                Input::SoftDrop,
            ],
            state: GameState::MakeNewShape,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.input_buffer, vec![Input::Pause]);
    }

    #[test]
    fn game_over_drains_input() {
        // Arrange a shape and a non-input state.
        let mut game = Game {
            input_buffer: vec![
                Input::Left,
                Input::Left,
                Input::Right,
                Input::Pause,
                Input::SoftDrop,
            ],
            state: GameState::GameOver,
            ..Default::default()
        };
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.input_buffer, vec![]);
    }
}
