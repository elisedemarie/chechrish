use crate::{COLS, Input, ROWS, shape::Shape};

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

    pub fn drop_shape(&mut self) {
        self.shape.as_mut().unwrap().move_down();
    }

    pub fn merge_shape(&mut self) {
        let shape = self.shape.take().unwrap();
        for cell in shape.get_cells() {
            self.set_cell(cell.y as usize, cell.x as usize, true);
        }
    }

    pub fn set_cell(&mut self, row: usize, col: usize, value: bool) {
        self.cells[row][col] = value; 
    }

    pub fn render_cells(&self) -> [[bool; COLS]; ROWS] {
        let mut cells_to_render = self.cells;
        // Include shape cells for render.
        if let Some(shape) = &self.shape {
            for cell in &shape.get_cells() {
                cells_to_render[cell.y as usize][cell.x as usize] = true; 
        }
        }
        cells_to_render
    }

    fn calculate_row_sum(&self, row_idx: usize) -> usize {
        self.cells[row_idx].iter().filter(|&&it| it).count()
    }
    
    pub fn check_rows(&mut self) {
        let mut row_idx = ROWS -1;
        while self.calculate_row_sum(row_idx) > 0 {
            if self.calculate_row_sum(row_idx) == COLS {
                self.cells[row_idx] = [false; COLS];
            } 
            row_idx += 1;
        }
    }

    pub fn transform_shape(&mut self, input: Option<Input>) {
        let Some(input) = input else {return};
        let Some(shape) = &mut self.shape else {return};
        match input {
            Input::Left => shape.move_left(),
            Input::Right => shape.move_right(),
            Input::SoftDrop => shape.move_down(),
            _ => ()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shape::{Orientation, ShapeType};

    #[test]
    fn new_board_is_all_empty() {}

    #[test]
    fn set_cell_marks_cell_as_filled() {}

    #[test]
    fn add_shape_makes_it_visible_in_render() {}

    #[test]
    fn render_cells_without_shape_returns_grid_only() {}

    #[test]
    fn drop_shape_increments_y_position() {}

    #[test]
    fn merge_shape_bakes_cells_into_grid() {}

    #[test]
    fn merge_shape_removes_active_shape() {}

    #[test]
    fn full_row_is_cleared_by_check_rows() {}

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

