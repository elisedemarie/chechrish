use crate::{COLS, ROWS, shape::Shape};

pub struct Board {
    shape: Option<Shape>,
    cells: [[bool; COLS]; ROWS],
}

impl Board {
    pub fn default() -> Self {
        Self {
            shape: None,
            cells: [[false; COLS]; ROWS],
        }
    }

    pub fn add_shape(&mut self, shape: Shape) {
        self.shape = Some(shape);
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: bool) {
        self.cells[row][col] = value; 
    }
}

