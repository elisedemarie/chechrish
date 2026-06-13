use crate::COLS;
use alloc::vec::Vec;
use alloc::vec;

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
}

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
                Position {x: 0, y: 0},
                Position {x: 0, y: 1},
                Position {x: 0, y: 2},
                Position {x: 0, y: 3},
            ],
            Self::T => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 2, y: 0},
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
                Position {x: 0, y: 0},
                Position {x: 0, y: 1},
                Position {x: 0, y: 2},
                Position {x: 1, y: 2},
            ],
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
        let shape_sells = self.shape_type.shape_cells();
        match self.orientation {
            Orientation::North => shape_sells,
            Orientation::East => rotate_shape(shape_sells),
            Orientation::South => rotate_shape(rotate_shape(shape_sells)),
            Orientation::West => rotate_shape(rotate_shape(rotate_shape(shape_sells)))
        }
    } 

}

fn rotate_shape(positions: Vec<Position>) -> Vec<Position> {
    positions.iter().map(|pos: &Position| {
        Position {
            x: pos.y,
            y: -pos.x
        }
    }).collect()
}
