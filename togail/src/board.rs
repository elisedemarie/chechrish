use crate::{
    COLS, ROWS,
    shape::{Orientation, Position, Shape, ShapeType},
};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum MoveOutcome {
    Moved,
    Failed,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Move {
    Drop,
    Left,
    Right,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Rotation {
    Clockwise,
    AntiClockwise,
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum SpawnOutcome {
    Spawned,
    FullBoard,
}

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
    #[cfg(test)]
    pub fn get_shape(&self) -> Option<Shape> {
        self.shape
    }

    #[cfg(test)]
    pub fn get_shape_pos(&self) -> Option<Position> {
        self.shape_position
    }

    pub fn add_new_shape(&mut self, shape_type: ShapeType) -> SpawnOutcome {
        let shape = Shape::new(shape_type, Orientation::North);
        let spawn_col = (COLS - shape.shape_type.shape_size()) / 2;
        let position = Position::new(spawn_col as isize, 0);
        let shape_cells = shape.cells_at(position);
        if self.is_valid_placement(&shape_cells) {
            self.shape = Some(shape);
            self.shape_position = Some(position);
            SpawnOutcome::Spawned
        } else {
            SpawnOutcome::FullBoard
        }
    }

    fn is_valid_placement(&self, positions: &[Position]) -> bool {
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
        true
    }

    pub fn move_shape(&mut self, input_move: Move) -> MoveOutcome {
        let vec = match input_move {
            Move::Left => Position::new(-1, 0),
            Move::Right => Position::new(1, 0),
            Move::Drop => Position::new(0, 1),
        };
        let pos = self
            .shape_position
            .expect("move_shape called with no shape position.");
        let shape = self.shape.expect("move_shape called with no shape.");
        let new_pos = pos + vec;
        let new_cells = shape.cells_at(new_pos);
        if self.is_valid_placement(&new_cells) {
            self.shape_position = Some(new_pos);
            MoveOutcome::Moved
        } else {
            MoveOutcome::Failed
        }
    }

    pub fn hard_drop(&mut self) {
        while self.move_shape(Move::Drop) == MoveOutcome::Moved {}
    }

    pub fn rotate_shape(&mut self, input_rotation: Rotation) -> MoveOutcome {
        let pos = self
            .shape_position
            .expect("move_shape called with no shape position.");
        let shape = self.shape.expect("move_shape called with no shape.");
        let mut new_shape = shape;
        match input_rotation {
            Rotation::Clockwise => new_shape.rotate_clockwise(),
            Rotation::AntiClockwise => new_shape.rotate_anti_clockwise(),
        };
        let new_cells = new_shape.cells_at(pos);
        if self.is_valid_placement(&new_cells) {
            self.shape = Some(new_shape);
            MoveOutcome::Moved
        } else {
            MoveOutcome::Failed
        }
    }

    pub fn merge_shape(&mut self) {
        let Some(shape) = self.shape else { return };
        let Some(pos) = self.shape_position else {
            return;
        };
        let cells = shape.cells_at(pos);
        for cell in cells {
            self.set_cell(cell.y as usize, cell.x as usize, true);
        }
        self.shape = None;
        self.shape_position = None;
    }

    fn set_cell(&mut self, row: usize, col: usize, value: bool) {
        self.cells[row][col] = value;
    }

    pub fn render_cells(&self) -> [[bool; COLS]; ROWS] {
        let mut cells_to_render = self.cells;
        if let Some(shape) = &self.shape
            && let Some(pos) = self.shape_position
        {
            for cell in shape.get_cells() {
                let pos_on_board = cell + pos;
                cells_to_render[pos_on_board.y as usize][pos_on_board.x as usize] = true;
            }
        }
        cells_to_render
    }

    pub fn check_rows(&mut self) -> u32 {
        let mut cleared_rows = 0;
        for row_idx in 0..ROWS {
            let row_sum = calc_row_sum(self.cells[row_idx]);
            if row_sum == COLS {
                self.cells[row_idx] = [false; COLS];
                self.cells[0..=row_idx].rotate_right(1);
                cleared_rows += 1
            }
        }
        cleared_rows
    }
}

fn calc_row_sum(row: [bool; COLS]) -> usize {
    row.iter().filter(|&&it| it).count()
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
        board.add_new_shape(ShapeType::Z);
        let cells = board.render_cells();
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn drop_shape_increments_y_position() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        let y_0 = board.shape_position.unwrap().y;
        board.move_shape(Move::Drop);
        let y_1 = board.shape_position.unwrap().y;
        assert_eq!(y_1, y_0 + 1);
    }

    #[test]
    fn merge_shape_sets_cells_to_true() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        board.merge_shape();
        let cells = board.cells;
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn merge_shape_removes_active_shape() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        board.merge_shape();
        assert!(board.shape.is_none());
    }

    #[test]
    fn full_row_is_cleared_by_check_rows() {
        let mut board = Board::default();
        board.cells[ROWS - 1] = [true; COLS];
        board.check_rows();
        let cells = board.cells;
        assert!(cells.iter().all(|col| col.iter().all(|cell| !*cell)))
    }

    #[test]
    fn check_rows_returns_zero_when_no_row_is_full() {
        let mut board = Board::default();
        let cleared = board.check_rows();
        assert_eq!(cleared, 0);
    }

    #[test]
    fn check_rows_returns_one_for_a_single_cleared_row() {
        let mut board = Board::default();
        board.cells[ROWS - 1] = [true; COLS];
        let cleared = board.check_rows();
        assert_eq!(cleared, 1);
    }

    #[test]
    fn check_rows_returns_count_for_multiple_cleared_rows() {
        let mut board = Board::default();
        board.cells[ROWS - 1] = [true; COLS];
        board.cells[ROWS - 2] = [true; COLS];
        board.cells[ROWS - 3] = [true; COLS];
        let cleared = board.check_rows();
        assert_eq!(cleared, 3);
    }

    #[test]
    fn partial_row_is_not_cleared_by_check_rows() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS - 1] = partial_row;
        board.check_rows();
        let cells = board.cells;
        assert!(cells.iter().any(|col| col.iter().any(|cell| *cell)))
    }

    #[test]
    fn cleared_row_drops_row_above() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS - 1] = [true; COLS];
        board.cells[ROWS - 2] = partial_row;
        board.check_rows();
        let cells = board.cells;
        assert_eq!(cells[ROWS - 1], partial_row);
    }

    #[test]
    fn multiple_cleared_row_drops_above_rows_fully() {
        let mut board = Board::default();
        let mut partial_row = [true; COLS];
        partial_row[1] = false;
        board.cells[ROWS - 1] = [true; COLS];
        board.cells[ROWS - 2] = [true; COLS];
        board.cells[ROWS - 3] = [true; COLS];
        board.cells[ROWS - 4] = partial_row;
        board.check_rows();
        let cells = board.cells;
        assert_eq!(cells[ROWS - 1], partial_row);
    }

    #[test]
    fn transform_left_moves_shape_left() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let x_0 = 2;
        let pos_0 = Position::new(x_0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(Move::Left);
        let x_1 = board.shape_position.unwrap().x;
        assert_eq!(x_1, x_0 - 1);
    }

    #[test]
    fn transform_right_moves_shape_right() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let x_0 = 2;
        let pos_0 = Position::new(x_0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(Move::Right);
        let x_1 = board.shape_position.unwrap().x;
        assert_eq!(x_1, x_0 + 1);
    }

    #[test]
    fn transform_soft_drop_moves_shape_down() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let y_0 = 0;
        let pos_0 = Position::new(2, y_0);
        board.shape = Some(shape);
        board.shape_position = Some(pos_0);
        board.move_shape(Move::Drop);
        let y_1 = board.shape_position.unwrap().y;
        assert_eq!(y_1, y_0 + 1);
    }

    #[test]
    fn shape_on_bottom_returns_landed_for_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, ROWS as isize - 2);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        let res = board.move_shape(Move::Drop);
        assert_eq!(res, MoveOutcome::Failed)
    }

    #[test]
    fn shape_not_on_bottom_returns_dropped_for_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        let res = board.move_shape(Move::Drop);
        assert_eq!(res, MoveOutcome::Moved)
    }

    #[test]
    fn shape_on_single_block_does_not_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        board.cells[2][1] = true;
        let res = board.move_shape(Move::Drop);
        assert_eq!(res, MoveOutcome::Failed)
    }

    #[test]
    fn shape_not_fully_on_single_block_does_drop() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(shape_pos);
        board.cells[2][0] = true;
        let res = board.move_shape(Move::Drop);
        assert_eq!(res, MoveOutcome::Moved)
    }

    #[test]
    fn hard_drop_on_empty_board_piece_cannot_drop_further() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.hard_drop();
        assert_eq!(board.move_shape(Move::Drop), MoveOutcome::Failed);
    }

    #[test]
    fn hard_drop_with_obstacle_piece_rests_on_top_of_it() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        // Z North at (0,y): bottom-right cell is (1, y+1). Blocking row 5 col 1
        // forces the piece to rest at y=3 (its (1,y+2) would be row 5).
        board.cells[5][1] = true;
        board.hard_drop();
        assert_eq!(board.get_shape_pos().unwrap().y, 3);
    }

    #[test]
    fn hard_drop_does_not_move_piece_into_existing_cells() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::O, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        // O piece (2×2) at x=0 occupies cols 0-1. Fill the bottom row at those cols.
        board.cells[ROWS - 1][0] = true;
        board.cells[ROWS - 1][1] = true;
        board.hard_drop();
        // bottom cells of the piece (row y+1) must be clear of the blocked row
        let y = board.get_shape_pos().unwrap().y;
        assert!((y + 1) as usize != ROWS - 1);
    }

    #[test]
    fn full_board_does_not_make_shape() {
        let mut board = Board::default();
        board.cells = [[true; COLS]; ROWS];
        board.add_new_shape(ShapeType::Z);
        assert!(board.shape.is_none());
        assert!(board.shape_position.is_none());
    }

    #[test]
    fn full_board_returns_full_board_on_make_new_shape() {
        let mut board = Board::default();
        board.cells = [[true; COLS]; ROWS];
        let res = board.add_new_shape(ShapeType::Z);
        assert_eq!(res, SpawnOutcome::FullBoard)
    }

    #[test]
    fn rotate_i_on_wall_rotates() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::I);
        board.shape_position = Some(Position::new(-2, 0));
        board.rotate_shape(Rotation::Clockwise);
        let obs_cells = board.render_cells();
        let mut exp_cells = [[false; COLS]; ROWS];
        exp_cells[2][0] = true;
        exp_cells[2][1] = true;
        exp_cells[2][2] = true;
        exp_cells[2][3] = true;
        assert_eq!(obs_cells, exp_cells);
    }

    #[test]
    fn rotate_east_z_on_wall_rotates() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::East);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(-1, 0));
        board.rotate_shape(Rotation::AntiClockwise);
        let obs_cells = board.render_cells();
        let mut exp_cells = [[false; COLS]; ROWS];
        exp_cells[0][1] = true;
        exp_cells[1][1] = true;
        exp_cells[1][2] = true;
        exp_cells[2][2] = true;
        assert_eq!(obs_cells, exp_cells);
    }
}
