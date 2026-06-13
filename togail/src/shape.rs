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
    shape_type: ShapeType,
    orientation: Orientation,
}

impl Shape {
    pub fn new(shape_type: ShapeType, orientation: Orientation) -> Self {
        Self {
            shape_type,
            orientation,
        }
    }
}
