use crate::ROWS;
use alloc::vec::Vec;
use alloc::vec;

pub struct Position {
    pub x: isize,
    pub y: isize,
}

impl Position {
    pub fn start_position() -> Self {
        Self {
            x: (ROWS / 2) as isize,
            y: 0,
        }
    }

    pub fn new(x: usize, y: usize) -> Self {
        Self {x, y}
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
        match self.shape_type {
            ShapeType::I => vec![
                Position {x: 0, y: 0},
                Position {x: 0, y: 1},
                Position {x: 0, y: 2},
                Position {x: 0, y: 3},
            ],
            ShapeType::T => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 2, y: 0},
                Position {x: 1, y: 1},
            ],
            ShapeType::O => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 0, y: 1},
                Position {x: 1, y: 1},
            ],
            ShapeType::Z => vec![
                Position {x: 0, y: 0},
                Position {x: 1, y: 0},
                Position {x: 1, y: 1},
                Position {x: 2, y: 1},
            ],
            ShapeType::S => vec![
                Position {x: 1, y: 0},
                Position {x: 2, y: 0},
                Position {x: 0, y: 1},
                Position {x: 1, y: 1},
            ],
            ShapeType::J => vec![
                Position {x: 1, y: 0},
                Position {x: 1, y: 1},
                Position {x: 1, y: 2},
                Position {x: 0, y: 2},
            ],
            ShapeType::L => vec![
                Position {x: 0, y: 0},
                Position {x: 0, y: 1},
                Position {x: 0, y: 2},
                Position {x: 1, y: 2},
            ],
        }
    } 
}
