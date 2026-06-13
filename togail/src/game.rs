use crate::{Frame, GRAVITY_TICK, Input, board::Board, shape::Shape};


#[derive(Debug, Clone, Copy)]
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
    pub fn new() -> Self {
        Self {
            board: Board::default(),
            state: GameState::MakeNewShape,
            clock: 0,
        }
    }

    fn make_new_shape(&mut self) {
        let new_shape = Shape::make_new_shape();
        self.board.add_shape(new_shape);
        self.state = GameState::TakeInput;
    }

    fn drop_shape(&mut self) {
        self.board.drop_shape();
        self.state = GameState::TakeInput;
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
        self.board.transform_shape(input)
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

    pub fn debug_get_state(&self) -> GameState {
        self.state
    }

    pub fn debug_get_clock(&self) -> u32 {
        self.clock
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_game_starts_in_make_new_shape_state() {}

    #[test]
    fn first_tick_spawns_a_shape() {}

    #[test]
    fn clock_accumulates_delta_ms_each_tick() {}

    #[test]
    fn shape_drops_after_gravity_tick_elapses() {}

    #[test]
    fn clock_resets_after_gravity_fires() {}

    #[test]
    fn input_moves_shape_left_during_take_input_state() {}

    #[test]
    fn input_moves_shape_right_during_take_input_state() {}
}
