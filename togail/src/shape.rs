use alloc::vec::Vec;
use alloc::vec;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub fn start_position() -> Self {
        Self {
            x: 0,
            y: 0,
        }
    }

    pub fn new(x: isize, y:isize) -> Self {
        Position {
            x, y
        }
    }
}

#[derive(Clone, Copy)]
pub enum ShapeType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl ShapeType {
    pub fn shape_cells(&self) -> Vec<Position> {
        match self {
            Self::I => vec![
                Position {x: 2, y: 0},
                Position {x: 2, y: 1},
                Position {x: 2, y: 2},
                Position {x: 2, y: 3},
            ],
            Self::T => vec![
                Position {x: 0, y: 1},
                Position {x: 1, y: 1},
                Position {x: 2, y: 1},
                Position {x: 1, y: 1},
            ],
            Self::O => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 0, y: 1},
                Position {x: 1, y: 1},
            ],
            Self::Z => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 1, y: 1},
                Position {x: 2, y: 1},
            ],
            Self::S => vec![
                Position {x: 1, y: 0},
                Position {x: 2, y: 0},
                Position {x: 0, y: 1},
                Position {x: 1, y: 1},
            ],
            Self::J => vec![
                Position {x: 1, y: 0},
                Position {x: 1, y: 1},
                Position {x: 1, y: 2},
                Position {x: 0, y: 2},
            ],
            Self::L => vec![
                Position {x: 1, y: 0},
                Position {x: 1, y: 1},
                Position {x: 1, y: 2},
                Position {x: 2, y: 2},
            ],
        }
    }

    pub fn shape_size(&self) -> usize {
        match self {
            ShapeType::O => 2,
            ShapeType::I => 4,
            _ => 3
        } 
    }
}

pub enum Orientation {
    North,
    East,
    South,
    West,
}

pub struct Shape {
    pub shape_type: ShapeType,
    pub orientation: Orientation,
    pub position: Position,
}

impl Shape {
    pub fn new(shape_type: ShapeType, orientation: Orientation) -> Self {
        Self {
            shape_type,
            orientation,
            position: Position::start_position(),
        }
    }

    pub fn get_cells(&self) -> Vec<Position> {
        let shape_cells = self.shape_type.shape_cells();
        match self.orientation {
            Orientation::North => shape_cells,
            Orientation::East => rotate_shape(shape_cells, self.shape_type),
            Orientation::South => rotate_shape(rotate_shape(shape_cells, self.shape_type), self.shape_type),
            Orientation::West => rotate_shape(rotate_shape(rotate_shape(shape_cells, self.shape_type), self.shape_type), self.shape_type)
        }
    } 

    pub fn make_new_shape() -> Self {
        // TODO make random
        Self {
            shape_type: ShapeType::Z,
            orientation: Orientation::North,
            position: Position::start_position()

        }
    }

    pub fn move_left(&mut self) {
        self.position.x -= 1;
    }

    pub fn move_right(&mut self) {
        self.position.x += 1;
    }

    pub fn move_down(&mut self) {
        self.position.y += 1;
    }

    pub fn rotate_clockwise(&mut self) {
        self.orientation = match self.orientation {
            Orientation::North => Orientation::East,
            Orientation::East => Orientation::South,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North
        }
    }

    pub fn rotate_anti_clockwise(&mut self) {
        self.orientation = match self.orientation {
            Orientation::South => Orientation::East,
            Orientation::West => Orientation::South,
            Orientation::North => Orientation::West,
            Orientation::East => Orientation::North
        }

    }
}

fn rotate_shape(positions: Vec<Position>, shape_type: ShapeType) -> Vec<Position> {
    positions.iter().map(|pos: &Position| {
        Position {
            x: pos.y,
            y: pos.x -1 + shape_type.shape_size() as isize
        }
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── cell counts ──────────────────────────────────────────────────────────

    #[test]
    fn i_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::I, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn o_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::O, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn t_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::T, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn s_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::S, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn z_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn j_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::J, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    #[test]
    fn l_piece_has_four_cells() {
        let shape = Shape::new(ShapeType::L, Orientation::North);
        let cells = shape.get_cells();
        assert_eq!(cells.len(), 4);
    }

    // ── north orientation ────────────────────────────────────────────────────

    #[test]
    fn i_piece_north_is_vertical_line() {
        let shape = Shape::new(ShapeType::I, Orientation::North);
        let expected_positions = vec![
            Position::new(2, 0),
            Position::new(2, 1),
            Position::new(2, 2),
            Position::new(2, 3),
        ];
        let obs = shape.get_cells();
        assert_eq!(obs.len(), expected_positions.len());
        assert!(expected_positions.iter().all(|it| obs.contains(it)));
    }

    // ── rotation ─────────────────────────────────────────────────────────────

    #[test]
    fn four_rotations_keep_shape_constant() {
        let shape = Shape::new(ShapeType::I, Orientation::North);
        let shape_cells = shape.get_cells();
        let new_cells = rotate_shape(rotate_shape(rotate_shape(rotate_shape(shape_cells.clone(), ShapeType::I), ShapeType::I), ShapeType::I), ShapeType::I);
        assert_eq!(new_cells.len(), shape_cells.len());
        assert!(shape_cells.iter().all(|it| new_cells.contains(it)));
        
    }

    #[test]
    fn rotating_i_east_gives_horizontal_line() {
        let shape = Shape::new(ShapeType::I, Orientation::East);
        let expected_positions = vec![
            Position::new(3, 1),
            Position::new(2, 1),
            Position::new(1, 1),
            Position::new(0, 1),
        ];
        let obs = shape.get_cells();
        assert_eq!(obs.len(), expected_positions.len());
        assert!(expected_positions.iter().all(|it| obs.contains(it)));
    }

    #[test]
    fn rotating_i_south_gives_vertical_line_again() {
        let shape = Shape::new(ShapeType::I, Orientation::South);
        let expected_positions = vec![
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(1, 3),
        ];
        let obs = shape.get_cells();
        assert_eq!(obs.len(), expected_positions.len());
        assert!(expected_positions.iter().all(|it| obs.contains(it)));
    }

    #[test]
    fn rotating_i_west_gives_horizontal_line_again() {
        let shape = Shape::new(ShapeType::I, Orientation::West);
        let expected_positions = vec![
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(3, 1),
        ];
        let obs = shape.get_cells();
        assert_eq!(obs.len(), expected_positions.len());
        assert!(expected_positions.iter().all(|it| obs.contains(it)));
    }

    #[test]
    fn o_piece_cells_unchanged_after_rotation() {}

    // ── movement ─────────────────────────────────────────────────────────────

    #[test]
    fn move_left_decrements_x() {}

    #[test]
    fn move_right_increments_x() {}

    #[test]
    fn move_down_increments_y() {}
}
