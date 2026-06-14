use crate::{Frame, GRAVITY_TICK, Input, board::Board, shape::Shape};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    MakeNewShape,
    MergeShape,
    DropShape,
    TakeInput,
    CheckRows,
}

pub struct Game {
    board: Board,
    state: GameState,
    clock: u32,
}

impl Game {
    pub fn default() -> Self {
        Self {
            board: Board::default(),
            state: GameState::MakeNewShape,
            clock: 0,
        }
    }

    fn make_new_shape(&mut self) {
        self.board.add_new_shape();
        self.state = GameState::TakeInput;
    }

    fn drop_shape(&mut self) {
        if self.board.drop_shape() {
            self.state = GameState::TakeInput;
        } else {
            self.state = GameState::MergeShape;
        };
    }

    fn merge_shape(&mut self) {
        self.board.merge_shape();
        self.state = GameState::CheckRows;
    }

    fn check_rows(&mut self) {
        self.board.check_rows();
        self.state = GameState::MakeNewShape;
    }

    fn take_input(&mut self, input: Option<Input>) {
        let Some(input) = input else { return };
        match input {
            Input::Left | Input::Right | Input::SoftDrop => self.board.move_shape(input),
            Input::RotateCw | Input::RotateCcw => self.board.rotate_shape(input),
            _ => ()
        }
    }

    fn step_state(&mut self, input: Option<Input>) {
        match self.state {
            GameState::DropShape => self.drop_shape(),
            GameState::MakeNewShape => self.make_new_shape(),
            GameState::CheckRows => self.check_rows(),
            GameState::TakeInput => self.take_input(input),
            GameState::MergeShape => self.merge_shape(),
        }
    }

    pub fn tick(&mut self, inputs: &[Input], delta_ms: u32) {
        // TODO this is a hack to just take the first input.
        let input = inputs.first().copied();
        self.clock += delta_ms;
        if self.clock > GRAVITY_TICK {
            self.state = GameState::DropShape;
            self.clock = 0;
        }
        self.step_state(input);
    }

    pub fn get_frame(&self) -> Frame {
        let buffer = &self.board.render_cells().clone();
        Frame {
            board: buffer.clone(),
            score: 0,
            level: 1,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

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
        assert!(game.clock > 0);
    }

    #[test]
    fn shape_drops_after_gravity_tick_elapses() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        let y_1 = game.board.get_shape_pos().clone().unwrap().y;
        game.tick(&inputs, GRAVITY_TICK);
        let y_2 = game.board.get_shape_pos().clone().unwrap().y;
        assert!(y_1 < y_2)
    }

    #[test]
    fn clock_resets_after_gravity_fires() {
        let mut game = Game::default();
        let inputs: [Input; 0] = [];
        game.tick(&inputs, GRAVITY_TICK + 1);
        assert_eq!(game.clock, 0);
    }

    #[test]
    fn make_new_shape_state_moves_to_take_input() {
        let mut game = Game::default();
        game.state = GameState::MakeNewShape;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn take_input_transitions_to_take_input() {
        let mut game = Game::default();
        game.state = GameState::TakeInput;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::TakeInput);
    }

    #[test]
    fn merge_shape_transitions_to_check_rows() {
        let mut game = Game::default();
        game.state = GameState::MergeShape;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::CheckRows);
    }

    #[test]
    fn check_rows_transitions_to_make_new_shape() {
        let mut game = Game::default();
        game.state = GameState::CheckRows;
        let inputs: [Input; 0] = [];
        game.tick(&inputs, 1);
        assert_eq!(game.state, GameState::MakeNewShape);
    }
}
