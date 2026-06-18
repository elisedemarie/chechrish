use crate::shape::ShapeType;

pub fn get_random_shape_type(state: &mut u64) -> ShapeType {
    let new_seed = make_new_state(*state);
    *state = new_seed;
    convert_state_to_shape_type(new_seed)
}

fn convert_state_to_shape_type(state: u64) -> ShapeType {
    match state % 7 {
        0 => ShapeType::I,
        1 => ShapeType::T,
        2 => ShapeType::Z,
        3 => ShapeType::S,
        4 => ShapeType::J,
        5 => ShapeType::L,
        6 => ShapeType::O,
        _ => unreachable!()
    }

}


fn make_new_state(state: u64) -> u64 {
    // xorshift64
    let mut x = state;
    if x == 0 {
        x += 1
    }
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_advances_after_call() {
        let mut state = 42;
        get_random_shape_type(&mut state);
        assert_ne!(state, 42);
    }

    #[test]
    fn same_seed_produces_same_sequence() {
        let mut state_a = 999;
        let mut state_b = 999;
        for _ in 0..20 {
            assert_eq!(
                get_random_shape_type(&mut state_a),
                get_random_shape_type(&mut state_b)
            );
        }
    }

    #[test]
    fn zero_seed_does_not_get_stuck() {
        let result = make_new_state(0);
        assert_ne!(result, 0);
    }

    #[test]
    fn all_shape_types_are_reachable() {
        let mut state = 1;
        let mut seen = [false; 7];
        for _ in 0..100 {
            let idx = match get_random_shape_type(&mut state) {
                ShapeType::I => 0,
                ShapeType::T => 1,
                ShapeType::Z => 2,
                ShapeType::S => 3,
                ShapeType::J => 4,
                ShapeType::L => 5,
                ShapeType::O => 6,
            };
            seen[idx] = true;
        }
        assert!(seen.iter().all(|&s| s));
    }
}
