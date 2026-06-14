use crate::{
    COLS, Input, ROWS,
    shape::{Orientation, Position, Shape, ShapeType},
};

pub struct Board {
    shape: Option<Shape>,
    shape_position: Option<Position>,
    cells: [[bool; COLS]; ROWS],
}

impl Board {
    pub fn new(shape: Option<Shape>, shape_position: Option<Position>) -> Self {
        let cells = [[false; COLS]; ROWS];
        Self {
            shape,
            shape_position,
            cells,
        }
    }

    pub fn default() -> Self {
        Self {
            shape: None,
            shape_position: None,
            cells: [[false; COLS]; ROWS],
        }
    }

    pub fn get_shape(&self) -> &Option<Shape> {
        &self.shape
    }

    pub fn get_shape_pos(&self) -> &Option<Position> {
        &self.shape_position
    }

    pub fn move_shape(&mut self, input: Input) {
        let increment = match input {
            Input::Left => -1,
            Input::Right => 1,
            _ => return,
        };
        let Some(pos) = self.shape_position else {
            return;
        };
        let Some(shape) = &mut self.shape else { return };
        let new_pos = Position {
            x: pos.x + increment,
            y: pos.y,
        };
        let shape_cells = shape.get_cells().clone();
        let new_cells = shape_cells.map(|pos| pos + new_pos);
        if self.check_collision(&new_cells) {
            self.shape_position = Some(new_pos)
        }
    }

    fn check_collision(&self, positions: &[Position]) -> bool {
        for pos in positions {
            if pos.x < 0
                || pos.x as usize >= COLS
                || pos.y < 0
                || pos.y as usize >= ROWS
                || self.cells[pos.y as usize][pos.x as usize]
            {
                return false;
            }
        }
        return true;
    }

    pub fn drop_shape(&mut self) -> bool {
        let Some(shape_position) = self.shape_position else {return true};
        let Some(shape) = self.shape else {return true};
        let mut new_pos = shape_position;
        new_pos.y += 1;
        let shape_cells = shape.get_cells().clone();
        let new_cells = shape_cells.map(|pos| pos + new_pos);
        if self.check_collision(&new_cells) {
            self.shape_position = Some(new_pos);
            return true
        } else {
            return false
        }
    }

    pub fn rotate_shape(&mut self, input: Input) {
        let Some(pos) = self.shape_position else {
            return;
        };
        let Some(shape) = self.shape else { return };
        let mut new_shape = shape.clone();
        match input {
            Input::RotateCw => new_shape.rotate_clockwise(),
            Input::RotateCcw => new_shape.rotate_anti_clockwise(),
            _ => return
        };
        let shape_cells = new_shape.get_cells().clone();
        let new_cells = shape_cells.map(|shape_pos| shape_pos + pos);
        if self.check_collision(&new_cells) {
            self.shape = Some(new_shape);
        }
    }

    pub fn add_new_shape(&mut self) {
        // TODO Make random
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let position = Position::new(0, 0);
        self.shape = Some(shape);
        self.shape_position = Some(position)
    }

    pub fn merge_shape(&mut self) {
        let Some(shape) = self.shape else {return};
        let Some(pos) = self.shape_position else {return};
        let cells = shape.get_cells().map(|shape_pos| shape_pos + pos);
        for cell in cells {
            self.set_cell(cell.y as usize, cell.x as usize, true);
        }
        self.shape = None;
        self.shape_position = None;
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: bool) {
        self.cells[row][col] = value;
    }

    pub fn render_cells(&self) -> [[bool; COLS]; ROWS] {
        let mut cells_to_render = self.cells;
        // Include shape cells for render.
        if let Some(shape) = &self.shape {
            if let Some(pos) = self.shape_position {
                for cell in shape.get_cells() {
                    let pos_on_board = cell + pos;
                    cells_to_render[pos_on_board.y as usize][pos_on_board.x as usize] = true;
                }
            };
        }
        cells_to_render
    }

    fn calculate_row_sum(&self, row_idx: usize) -> usize {
        self.cells[row_idx].iter().filter(|&&it| it).count()
    }

    pub fn check_rows(&mut self) {
        let mut row_idx = ROWS - 1;
        while self.calculate_row_sum(row_idx) > 0 {
            if self.calculate_row_sum(row_idx) == COLS {
                self.cells[row_idx] = [false; COLS];
            }
            row_idx += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Orientation, ShapeType};

    #[test]
    fn new_board_is_all_empty() {
        let new_board = Board::default();
        let cells = new_board.cells;
        assert!(cells.iter().all(|col| col.iter().all(|cell| !*cell)))
    }

    #[test]
    fn set_cell_marks_cell_as_filled() {
        let row = 2;
        let col = 3;
        let mut new_board = Board::default();
        new_board.set_cell(row, col, true);
        assert!(new_board.cells[row][col])
    }

    #[test]
    fn add_shape_makes_it_visible_in_render() {
        let mut board = Board::default();
        board.add_new_shape();
        let cells = board.render_cells();
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn drop_shape_increments_y_position() {
        let mut board = Board::default();
        board.add_new_shape();
        let y_0 = board.shape_position.unwrap().y;
        board.drop_shape();
        let y_1 = board.shape_position.unwrap().y;
        assert_eq!(y_1, y_0 + 1);
    }

    #[test]
    fn merge_shape_sets_cells_to_true() {
        let mut board = Board::default();
        board.add_new_shape();
        board.merge_shape();
        let cells = board.cells;
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn merge_shape_removes_active_shape() {
        let mut board = Board::default();
        board.add_new_shape();
        board.merge_shape();
        assert!(board.shape.is_none());
    }

    #[test]
    fn full_row_is_cleared_by_check_rows() {
        let row_idx = 3;
        let mut board = Board::default();
        board.cells[row_idx] = [true; COLS];
        board.check_rows();
        let cells = board.cells;
        assert!(cells.iter().all(|col| col.iter().all(|cell| !*cell)))
    }

    #[test]
    fn partial_row_is_not_cleared_by_check_rows() {}

    #[test]
    fn transform_left_moves_shape_left() {}

    #[test]
    fn transform_right_moves_shape_right() {}

    #[test]
    fn transform_soft_drop_moves_shape_down() {}

    #[test]
    fn transform_with_no_input_does_nothing() {}

    #[test]
    fn transform_with_no_shape_does_nothing() {}
}
