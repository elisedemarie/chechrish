use crate::{
    COLS, Input, ROWS,
    shape::{Orientation, Position, Shape, ShapeType},
};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum DropOutcome { Dropped, Landed }

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum SpawnOutcome { Spawned, FullBoard }

pub struct Board {
    shape: Option<Shape>,
    shape_position: Option<Position>,
    cells: [[bool; COLS]; ROWS],
}

impl Default for Board {
    fn default() -> Self {
        Self {
            shape: None,
            shape_position: None,
            cells: [[false; COLS]; ROWS],
        }
    }

}

impl Board {
    pub fn get_shape(&self) -> &Option<Shape> {
        &self.shape
    }

    pub fn get_shape_pos(&self) -> &Option<Position> {
        &self.shape_position
    }

    pub fn add_new_shape(&mut self) -> SpawnOutcome {
        // TODO Make random
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let position = Position::new(0, 0);
        let shape_cells = shape.get_cells().map(|it| it + position);
        if self.check_collision(&shape_cells) {
            self.shape = Some(shape);
            self.shape_position = Some(position);
            SpawnOutcome::Spawned
        } else {
            SpawnOutcome::FullBoard
        }
    }

    pub fn move_shape(&mut self, input: Input) {
        let vec = match input {
            Input::Left => Position::new(-1, 0),
            Input::Right => Position::new(1, 0),
            Input::SoftDrop => Position::new(0, 1),
            _ => return,
        };
        let Some(pos) = self.shape_position else {
            return;
        };
        let Some(shape) = &mut self.shape else { return };
        let new_pos = pos + vec;
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

    pub fn drop_shape(&mut self) -> DropOutcome {
        let shape_position = self.shape_position.expect("There should be a shape position here!");
        let shape = self.shape.expect("There should be a shape here!");
        let mut new_pos = shape_position;
        new_pos.y += 1;
        let shape_cells = shape.get_cells().clone();
        let new_cells = shape_cells.map(|pos| pos + new_pos);
        if self.check_collision(&new_cells) {
            self.shape_position = Some(new_pos);
            DropOutcome::Dropped
        } else {
            DropOutcome::Landed
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

    pub fn check_rows(&mut self) {
        let mut row_idx = 0;
        while row_idx < ROWS {
            let row_sum = calc_row_sum(self.cells[row_idx]);
            if row_sum == COLS {
                self.cells[row_idx] = [false; COLS]; 
                self.cells[0..=row_idx].rotate_right(1);
            }
            row_idx += 1
        } 
    }
}

fn calc_row_sum(row: [bool; COLS]) -> usize {
    row.iter().filter(|&&it| it).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{board::{DropOutcome::Dropped, SpawnOutcome::FullBoard}, shape::{Orientation, ShapeType}};

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
        let mut board = Board::default();
        board.cells[ROWS-1] = [true; COLS];
        board.check_rows();
        let cells = board.cells;
        assert!(cells.iter().all(|col| col.iter().all(|cell| !*cell)))
    }

    #[test]
    fn partial_row_is_not_cleared_by_check_rows() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS-1] = partial_row.clone();
        board.check_rows();
        let cells = board.cells;
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn cleared_row_drops_row_above() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS-1] = [true; COLS];
        board.cells[ROWS-2] = partial_row.clone();
        board.check_rows();
        let cells = board.cells;
        assert_eq!(cells[ROWS-1], partial_row);
    }

    #[test]
    fn multiple_cleared_row_drops_above_rows_fully() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS-1] = [true; COLS];
        board.cells[ROWS-2] = [true; COLS];
        board.cells[ROWS-3] = [true; COLS];
        board.cells[ROWS-4] = partial_row.clone();
        board.check_rows();
        let cells = board.cells;
        assert_eq!(cells[ROWS-1], partial_row);
    }

    #[test]
    fn transform_left_moves_shape_left() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let x_0 = 2;
        let pos_0 = Position::new(x_0, 0);
        let input = Input::Left;
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(input);
        let x_1 = board.shape_position.unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn transform_right_moves_shape_right() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let x_0 = 2;
        let pos_0 = Position::new(x_0, 0);
        let input = Input::Right;
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(input);
        let x_1 = board.shape_position.unwrap().x;
        assert_eq!(x_1, x_0 + 1);
    }

    #[test]
    fn transform_soft_drop_moves_shape_down() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let y_0 = 0;
        let pos_0 = Position::new(2, y_0);
        let input = Input::SoftDrop;
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(input);
        let y_1 = board.shape_position.unwrap().y;
        assert_eq!(y_1, y_0 + 1);
    }

    #[test]
    fn transform_with_no_shape_does_nothing() {
        let mut board = Board::default();
        let input = Input::Left;
        board.move_shape(input);
        let cells = board.cells;
        assert!(cells.iter().all(|col| col.iter().all(|cell| !*cell)))
    }

    #[test]
    fn shape_on_bottom_returns_landed_for_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, ROWS as isize-2);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        let res = board.drop_shape();
        assert_eq!(res, DropOutcome::Landed)
    }

    #[test]
    fn shape_not_on_bottom_returns_dropped_for_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        let res = board.drop_shape();
        assert_eq!(res, DropOutcome::Dropped)
    }

    #[test]
    fn shape_on_single_block_does_not_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        board.cells[2][1] = true;
        let res = board.drop_shape();
        assert_eq!(res, DropOutcome::Landed)
    }

    #[test]
    fn shape_not_fully_on_single_block_does_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        board.cells[2][0] = true;
        let res = board.drop_shape();
        assert_eq!(res, DropOutcome::Dropped)
    }

    #[test]
    fn full_board_does_not_make_shape() {
        let mut board = Board::default();
        board.cells = [[true; COLS]; ROWS];
        board.add_new_shape(); 
        assert!(board.shape.is_none());
        assert!(board.shape_position.is_none());
    }

    #[test]
    fn full_board_returns_full_board_on_make_new_shape() {
        let mut board = Board::default();
        board.cells = [[true; COLS]; ROWS];
        let res = board.add_new_shape(); 
        assert_eq!(res, SpawnOutcome::FullBoard)
    }
}
