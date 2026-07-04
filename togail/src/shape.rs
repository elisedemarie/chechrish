use core::ops::Add;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }
}

impl Add for Position {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Position {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

type ShapeCells = [Position; 4];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    pub fn shape_cells(&self) -> ShapeCells {
        match self {
            Self::I => [
                Position { x: 2, y: 0 },
                Position { x: 2, y: 1 },
                Position { x: 2, y: 2 },
                Position { x: 2, y: 3 },
            ],
            Self::T => [
                Position { x: 0, y: 1 },
                Position { x: 1, y: 1 },
                Position { x: 2, y: 1 },
                Position { x: 1, y: 0 },
            ],
            Self::O => [
                Position { x: 0, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 0, y: 1 },
                Position { x: 1, y: 1 },
            ],
            Self::Z => [
                Position { x: 0, y: 0 },
                Position { x: 1, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 2, y: 1 },
            ],
            Self::S => [
                Position { x: 1, y: 0 },
                Position { x: 2, y: 0 },
                Position { x: 0, y: 1 },
                Position { x: 1, y: 1 },
            ],
            Self::J => [
                Position { x: 1, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 2 },
                Position { x: 0, y: 2 },
            ],
            Self::L => [
                Position { x: 1, y: 0 },
                Position { x: 1, y: 1 },
                Position { x: 1, y: 2 },
                Position { x: 2, y: 2 },
            ],
        }
    }

    pub fn shape_size(&self) -> usize {
        match self {
            ShapeType::O => 2,
            ShapeType::I => 4,
            _ => 3,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Orientation {
    North,
    East,
    South,
    West,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Shape {
    pub shape_type: ShapeType,
    pub orientation: Orientation,
}

impl Shape {
    pub fn new(shape_type: ShapeType, orientation: Orientation) -> Self {
        Self {
            shape_type,
            orientation,
        }
    }

    pub fn get_cells(&self) -> ShapeCells {
        let shape_cells = self.shape_type.shape_cells();
        let shape_size = self.shape_type.shape_size();
        match self.orientation {
            Orientation::North => shape_cells,
            Orientation::East => rotate_shape(shape_cells, shape_size),
            Orientation::South => rotate_shape(rotate_shape(shape_cells, shape_size), shape_size),
            Orientation::West => rotate_shape(
                rotate_shape(rotate_shape(shape_cells, shape_size), shape_size),
                shape_size,
            ),
        }
    }

    pub fn cells_at(&self, pos: Position) -> ShapeCells {
        self.get_cells().map(|cell| cell + pos)
    }

    pub fn rotate_clockwise(&mut self) {
        self.orientation = match self.orientation {
            Orientation::North => Orientation::East,
            Orientation::East => Orientation::South,
            Orientation::South => Orientation::West,
            Orientation::West => Orientation::North,
        }
    }

    pub fn rotate_anti_clockwise(&mut self) {
        self.orientation = match self.orientation {
            Orientation::South => Orientation::East,
            Orientation::West => Orientation::South,
            Orientation::North => Orientation::West,
            Orientation::East => Orientation::North,
        }
    }
}

fn rotate_shape(positions: ShapeCells, shape_size: usize) -> ShapeCells {
    positions.map(|pos: Position| Position {
        x: -pos.y - 1 + shape_size as isize,
        y: pos.x,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── cell counts ──────────────────────────────────────────────────────────

    fn assert_positions_equal(a: &[Position], b: &[Position]) {
        assert_eq!(a.len(), b.len());
        assert!(b.iter().all(|it| a.contains(it)));
    }

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
        let expected_positions = [
            Position::new(2, 0),
            Position::new(2, 1),
            Position::new(2, 2),
            Position::new(2, 3),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn o_piece_north_is_two_by_two_square() {
        let shape = Shape::new(ShapeType::O, Orientation::North);
        let expected_positions = [
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(0, 1),
            Position::new(1, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn t_piece_north_is_t_shape() {
        let shape = Shape::new(ShapeType::T, Orientation::North);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn s_piece_north_is_s_shape() {
        let shape = Shape::new(ShapeType::S, Orientation::North);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(2, 0),
            Position::new(0, 1),
            Position::new(1, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn z_piece_north_is_z_shape() {
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let expected_positions = [
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn j_piece_north_is_j_shape() {
        let shape = Shape::new(ShapeType::J, Orientation::North);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(0, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    #[test]
    fn l_piece_north_is_l_shape() {
        let shape = Shape::new(ShapeType::L, Orientation::North);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected_positions);
    }

    // ── rotation ─────────────────────────────────────────────────────────────

    #[test]
    fn rotating_i_four_times_does_not_change_shape() {
        let shape = Shape::new(ShapeType::I, Orientation::North);
        let shape_cells = shape.get_cells();
        let shape_size = ShapeType::I.shape_size();
        let new_cells = rotate_shape(
            rotate_shape(
                rotate_shape(rotate_shape(shape_cells, shape_size), shape_size),
                shape_size,
            ),
            shape_size,
        );
        assert_positions_equal(&new_cells, &shape_cells);
    }

    #[test]
    fn rotating_i_east_gives_horizontal_line() {
        let shape = Shape::new(ShapeType::I, Orientation::East);
        let expected_positions = [
            Position::new(3, 2),
            Position::new(2, 2),
            Position::new(1, 2),
            Position::new(0, 2),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_i_south_gives_vertical_line_again() {
        let shape = Shape::new(ShapeType::I, Orientation::South);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(1, 3),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_i_west_gives_horizontal_line_again() {
        let shape = Shape::new(ShapeType::I, Orientation::West);
        let expected_positions = [
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(3, 1),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_o_four_times_does_not_change_shape_any_time() {
        let shape_type = ShapeType::O;
        let shape_size = shape_type.shape_size();
        let shape = Shape::new(shape_type, Orientation::North);
        let shape_cells = shape.get_cells();
        let first_rotation = rotate_shape(shape_cells, shape_size);
        let second_rotation = rotate_shape(first_rotation, shape_size);
        let third_rotation = rotate_shape(second_rotation, shape_size);
        let forth_rotation = rotate_shape(third_rotation, shape_size);
        assert_positions_equal(&first_rotation, &shape_cells);
        assert_positions_equal(&second_rotation, &first_rotation);
        assert_positions_equal(&third_rotation, &second_rotation);
        assert_positions_equal(&forth_rotation, &third_rotation);
    }

    #[test]
    fn rotating_z_four_times_does_not_change_shape() {
        let shape_type = ShapeType::Z;
        let shape_size = shape_type.shape_size();
        let shape = Shape::new(ShapeType::Z, Orientation::North);
        let shape_cells = shape.get_cells();
        let new_cells = rotate_shape(
            rotate_shape(
                rotate_shape(rotate_shape(shape_cells, shape_size), shape_size),
                shape_size,
            ),
            shape_size,
        );
        assert_positions_equal(&new_cells, &shape_cells);
    }

    #[test]
    fn rotating_z_east_gives_right_cells() {
        let shape_type = ShapeType::Z;
        let shape = Shape::new(shape_type, Orientation::East);
        let expected_positions = [
            Position::new(2, 0),
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(1, 2),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_z_south_gives_right_cells() {
        let shape_type = ShapeType::Z;
        let shape = Shape::new(shape_type, Orientation::South);
        let expected_positions = [
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 2),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_z_west_gives_right_cells() {
        let shape_type = ShapeType::Z;
        let shape = Shape::new(shape_type, Orientation::West);
        let expected_positions = [
            Position::new(1, 0),
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(0, 2),
        ];
        let obs = shape.get_cells();
        assert_positions_equal(&obs, &expected_positions);
    }

    #[test]
    fn rotating_s_east_gives_right_cells() {
        let shape = Shape::new(ShapeType::S, Orientation::East);
        let expected = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(2, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_s_south_gives_right_cells() {
        let shape = Shape::new(ShapeType::S, Orientation::South);
        let expected = [
            Position::new(0, 2),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_s_west_gives_right_cells() {
        let shape = Shape::new(ShapeType::S, Orientation::West);
        let expected = [
            Position::new(0, 0),
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(1, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_t_east_gives_right_cells() {
        let shape = Shape::new(ShapeType::T, Orientation::East);
        let expected = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_t_south_gives_right_cells() {
        let shape = Shape::new(ShapeType::T, Orientation::South);
        let expected = [
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_t_west_gives_right_cells() {
        let shape = Shape::new(ShapeType::T, Orientation::West);
        let expected = [
            Position::new(0, 1),
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_j_east_gives_right_cells() {
        let shape = Shape::new(ShapeType::J, Orientation::East);
        let expected = [
            Position::new(0, 0),
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_j_south_gives_right_cells() {
        let shape = Shape::new(ShapeType::J, Orientation::South);
        let expected = [
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
            Position::new(2, 0),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_j_west_gives_right_cells() {
        let shape = Shape::new(ShapeType::J, Orientation::West);
        let expected = [
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 1),
            Position::new(2, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_l_east_gives_right_cells() {
        let shape = Shape::new(ShapeType::L, Orientation::East);
        let expected = [
            Position::new(0, 1),
            Position::new(0, 2),
            Position::new(1, 1),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_l_south_gives_right_cells() {
        let shape = Shape::new(ShapeType::L, Orientation::South);
        let expected = [
            Position::new(0, 0),
            Position::new(1, 0),
            Position::new(1, 1),
            Position::new(1, 2),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }

    #[test]
    fn rotating_l_west_gives_right_cells() {
        let shape = Shape::new(ShapeType::L, Orientation::West);
        let expected = [
            Position::new(0, 1),
            Position::new(1, 1),
            Position::new(2, 0),
            Position::new(2, 1),
        ];
        assert_positions_equal(&shape.get_cells(), &expected);
    }
}
