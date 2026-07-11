use core::iter::once;

use crate::{
    COLS, ROWS,
    shape::{Orientation, Position, Shape, ShapeType},
};

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
enum MoveOutcome {
    Moved(Position),
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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum DropOutcome {
    Dropped,
    Landed,
}

#[derive(Eq, PartialEq, Clone, Debug, Default)]
pub struct Board {
    shape: Option<Shape>,
    shape_position: Option<Position>,
    ghost_position: Option<Position>,
    cells: [[bool; COLS]; ROWS],
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
            self.update_ghost_position(position);
            SpawnOutcome::Spawned
        } else {
            SpawnOutcome::FullBoard
        }
    }

    fn is_valid_placement(&self, positions: &[Position]) -> bool {
        positions.iter().all(|pos| {
            (0..COLS as isize).contains(&pos.x)
                && (0..ROWS as isize).contains(&pos.y)
                && !self.cells[pos.y as usize][pos.x as usize]
        })
    }

    fn update_ghost_position(&mut self, new_pos: Position) {
        let ghost_pos = self.hard_drop_from_pos(new_pos);
        self.ghost_position = Some(ghost_pos)
    }

    pub fn get_ghost(&self) -> Option<[(usize, usize); 4]> {
        Some(
            self.shape?
                .cells_at(self.ghost_position?)
                .map(|it| (it.x as usize, it.y as usize)),
        )
    }

    pub fn move_shape(&mut self, input: Move) {
        let pos = self
            .shape_position
            .expect("transform_shape called without shape position.");
        if let MoveOutcome::Moved(new_pos) = self.move_shape_from_pos(input, pos) {
            self.shape_position = Some(new_pos);
            self.update_ghost_position(new_pos);
        }
    }

    fn move_shape_from_pos(&self, input_move: Move, pos: Position) -> MoveOutcome {
        let vec = match input_move {
            Move::Left => Position::new(-1, 0),
            Move::Right => Position::new(1, 0),
            Move::Drop => Position::new(0, 1),
        };
        let shape = self
            .shape
            .expect("move_shape_from_pos called with no shape.");
        let new_pos = pos + vec;
        let new_cells = shape.cells_at(new_pos);
        if self.is_valid_placement(&new_cells) {
            MoveOutcome::Moved(new_pos)
        } else {
            MoveOutcome::Failed
        }
    }

    pub fn hard_drop(&mut self) {
        let pos = self
            .shape_position
            .expect("hard_drop was called without there being a shape position.");
        self.shape_position = Some(self.hard_drop_from_pos(pos));
        self.ghost_position = None;
    }

    fn hard_drop_from_pos(&self, mut pos: Position) -> Position {
        while let MoveOutcome::Moved(next_pos) = self.move_shape_from_pos(Move::Drop, pos) {
            pos = next_pos;
        }
        pos
    }

    pub fn rotate_shape(&mut self, input_rotation: Rotation) {
        let pos = self
            .shape_position
            .expect("rotate_shape called with no shape position.");
        let shape = self.shape.expect("rotate_shape called with no shape.");
        let mut new_shape = shape;
        match input_rotation {
            Rotation::Clockwise => new_shape.rotate_clockwise(),
            Rotation::AntiClockwise => new_shape.rotate_anti_clockwise(),
        };
        let max_offset = shape.shape_type.shape_size() - 2;
        let step_sequence =
            once(0).chain((1..=max_offset).flat_map(|i| [-(i as isize), i as isize]));
        for step in step_sequence {
            let mut new_pos = pos;
            new_pos.x += step;
            let new_cells = new_shape.cells_at(new_pos);
            if self.is_valid_placement(&new_cells) {
                self.shape = Some(new_shape);
                self.shape_position = Some(new_pos);
                self.update_ghost_position(new_pos);
                return;
            };
        }
    }

    pub fn drop_shape(&mut self) -> DropOutcome {
        let pos = self
            .shape_position
            .expect("drop_shape called without shape position.");
        if let MoveOutcome::Moved(new_pos) = self.move_shape_from_pos(Move::Drop, pos) {
            self.shape_position = Some(new_pos);
            self.update_ghost_position(new_pos);
            DropOutcome::Dropped
        } else {
            DropOutcome::Landed
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
        self.ghost_position = None;
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
    fn hard_drop_on_empty_board_piece_cannot_drop_further() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let pos = Position::new(0, 0);
        board.shape = Some(shape);
        board.shape_position = Some(pos);
        board.hard_drop();
        assert_eq!(board.drop_shape(), DropOutcome::Landed);
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
        let mut board = Board {
            cells: [[true; COLS]; ROWS],
            ..Default::default()
        };
        board.add_new_shape(ShapeType::Z);
        assert!(board.shape.is_none());
        assert!(board.shape_position.is_none());
    }

    #[test]
    fn full_board_returns_full_board_on_make_new_shape() {
        let mut board = Board {
            cells: [[true; COLS]; ROWS],
            ..Default::default()
        };
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
        // East at (-1,0) → CCW → North at (-1,0) puts a cell at x=-1.
        // +1 kick: North at (0,0) is valid.
        exp_cells[0][0] = true;
        exp_cells[0][1] = true;
        exp_cells[1][1] = true;
        exp_cells[1][2] = true;
        assert_eq!(obs_cells, exp_cells);
    }

    // ── wall kick ────────────────────────────────────────────────────────────

    #[test]
    fn rotate_cw_near_left_wall_kicks_right_by_one() {
        // Z East at (-1,0) is valid. CW→Z South at (-1,0) puts a cell at x=-1.
        // +1 kick: Z South at (0,0) is valid.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::East);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(-1, 0));
        board.rotate_shape(Rotation::Clockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(0, 0));
    }

    #[test]
    fn rotate_ccw_near_left_wall_kicks_right_by_one() {
        // Z East at (-1,0) is valid. CCW→Z North at (-1,0) puts a cell at x=-1.
        // +1 kick: Z North at (0,0) is valid.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::East);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(-1, 0));
        board.rotate_shape(Rotation::AntiClockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(0, 0));
    }

    #[test]
    fn rotate_i_cw_near_right_wall_kicks_left_by_one() {
        // I North at (7,0): cells at x=9, valid. CW→I East at (7,0) puts a cell at x=10.
        // -1 kick: I East at (6,0) puts cells at x=6..9, valid.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::I, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(7, 0));
        board.rotate_shape(Rotation::Clockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(6, 0));
    }

    #[test]
    fn rotate_i_cw_near_right_wall_kicks_left_by_two() {
        // I South at (8,0): cells at x=9, valid. CW→I West at (8,0) puts cells at x=10,11.
        // -1 kick: I West at (7,0) still puts a cell at x=10.
        // -2 kick: I West at (6,0) puts cells at x=6..9, valid.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::I, Orientation::South);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(8, 0));
        board.rotate_shape(Rotation::Clockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(6, 0));
    }

    #[test]
    fn rotate_cw_near_right_wall_kicks_left_by_one() {
        // Z West at (8,0): cells at x=8,9, valid. CW→Z North at (8,0) puts a cell at x=10.
        // -1 kick: Z North at (7,0) puts cells at x=7..9, valid.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::West);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(8, 0));
        board.rotate_shape(Rotation::Clockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(7, 0));
    }

    #[test]
    fn rotation_fails_when_all_kick_positions_are_blocked() {
        // I North at (-2,0), CW→I East. Block entire row 2 so every kick position fails.
        // I East cells are always in row 2 — any x offset still hits blocked cells.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::I, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(-2, 0));
        board.cells[2] = [true; COLS];
        board.rotate_shape(Rotation::Clockwise);
        assert_eq!(board.shape_position.unwrap(), Position::new(-2, 0));
        assert_eq!(board.shape.unwrap().orientation, Orientation::North);
    }

    // ── ghost piece ──────────────────────────────────────────────────────────

    #[test]
    fn ghost_is_set_when_shape_spawns() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        assert!(board.ghost_position.is_some());
    }

    #[test]
    fn ghost_position_y_is_at_or_below_shape_y() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        let shape_y = board.shape_position.unwrap().y;
        let ghost_y = board.ghost_position.unwrap().y;
        assert!(ghost_y >= shape_y);
    }

    #[test]
    fn ghost_is_at_bottom_on_empty_board() {
        // Z North bottom cells are at y+1, so ghost lands at ROWS-2
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.update_ghost_position(Position::new(0, 0));
        assert_eq!(board.ghost_position.unwrap().y, ROWS as isize - 2);
    }

    #[test]
    fn ghost_x_matches_shape_x_on_empty_board() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(3, 0));
        board.update_ghost_position(Position::new(3, 0));
        assert_eq!(board.ghost_position.unwrap().x, 3);
    }

    #[test]
    fn ghost_equals_shape_position_when_piece_is_at_bottom() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let pos = Position::new(0, ROWS as isize - 2);
        board.shape = Some(shape);
        board.shape_position = Some(pos);
        board.update_ghost_position(pos);
        assert_eq!(board.ghost_position.unwrap(), pos);
    }

    #[test]
    fn ghost_lands_on_top_of_obstacle() {
        // Z North right-bottom cell is at (x+1, y+1). Obstacle at row 5 col 1
        // means piece can only reach y=3 before (1, y+1+1) hits row 5.
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.cells[5][1] = true;
        board.update_ghost_position(Position::new(0, 0));
        assert_eq!(board.ghost_position.unwrap().y, 3);
    }

    #[test]
    fn ghost_updates_x_when_shape_moves_left() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(3, 0));
        board.update_ghost_position(Position::new(3, 0));
        let ghost_x_before = board.ghost_position.unwrap().x;
        board.move_shape(Move::Left);
        let ghost_x_after = board.ghost_position.unwrap().x;
        assert_eq!(ghost_x_after, ghost_x_before - 1);
    }

    #[test]
    fn ghost_updates_x_when_shape_moves_right() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(3, 0));
        board.update_ghost_position(Position::new(3, 0));
        let ghost_x_before = board.ghost_position.unwrap().x;
        board.move_shape(Move::Right);
        let ghost_x_after = board.ghost_position.unwrap().x;
        assert_eq!(ghost_x_after, ghost_x_before + 1);
    }

    #[test]
    fn ghost_does_not_update_when_move_fails_at_wall() {
        // Z at x=0, moving left should fail
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.update_ghost_position(Position::new(0, 0));
        let ghost_before = board.ghost_position.unwrap();
        board.move_shape(Move::Left);
        assert_eq!(board.ghost_position.unwrap(), ghost_before);
    }

    #[test]
    fn ghost_updates_after_rotation() {
        // I North: bottommost cell at y+3, ghost y = ROWS-4
        // I East: bottommost cell at y+2, ghost y = ROWS-3
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::I, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.update_ghost_position(Position::new(0, 0));
        let ghost_y_before = board.ghost_position.unwrap().y;
        board.rotate_shape(Rotation::Clockwise);
        let ghost_y_after = board.ghost_position.unwrap().y;
        assert_ne!(ghost_y_after, ghost_y_before);
    }

    #[test]
    fn ghost_is_cleared_when_shape_merges() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        board.merge_shape();
        assert!(board.ghost_position.is_none());
    }

    #[test]
    fn ghost_is_none_with_no_active_shape() {
        let board = Board::default();
        assert!(board.ghost_position.is_none());
    }

    #[test]
    fn get_ghost_returns_none_with_no_shape() {
        let board = Board::default();
        assert!(board.get_ghost().is_none());
    }

    #[test]
    fn get_ghost_returns_four_cells_when_shape_exists() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        assert!(board.get_ghost().is_some());
        assert_eq!(board.get_ghost().unwrap().len(), 4);
    }

    #[test]
    fn get_ghost_cells_are_within_board_bounds() {
        let mut board = Board::default();
        board.add_new_shape(ShapeType::Z);
        let cells = board.get_ghost().unwrap();
        for (x, y) in cells {
            assert!(x < COLS, "ghost cell x={x} out of bounds");
            assert!(y < ROWS, "ghost cell y={y} out of bounds");
        }
    }

    #[test]
    fn get_ghost_cells_match_ghost_position() {
        // Z North at (0, ROWS-2): cells (0,ROWS-2),(1,ROWS-2),(1,ROWS-1),(2,ROWS-1)
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let pos = Position::new(0, ROWS as isize - 2);
        board.shape = Some(shape);
        board.shape_position = Some(pos);
        board.update_ghost_position(pos);
        let cells = board.get_ghost().unwrap();
        assert!(cells.contains(&(0, ROWS - 2)));
        assert!(cells.contains(&(1, ROWS - 2)));
        assert!(cells.contains(&(1, ROWS - 1)));
        assert!(cells.contains(&(2, ROWS - 1)));
    }

    #[test]
    fn ghost_position_after_hard_drop_equals_shape_position() {
        let mut board = Board::default();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        board.shape = Some(shape);
        board.shape_position = Some(Position::new(0, 0));
        board.update_ghost_position(Position::new(0, 0));
        let ghost_pos = board.ghost_position.unwrap();
        board.hard_drop();
        assert_eq!(board.shape_position.unwrap(), ghost_pos);
    }
}
