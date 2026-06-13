use crate::{Input, board::Board, shape::Shape};

enum GameState {
    MakeNewShape,
    MergeShape,
    DropShape,
    TakeInput,
    CheckRows,
}

impl GameState {
}

pub struct Game {
    board: Board,
    state: GameState, 
}

impl Game {
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
}
