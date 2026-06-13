use crate::ROWS;

type Position = [usize; 2];

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
            position: [ROWS / 2, 0],
        }
    }
}
