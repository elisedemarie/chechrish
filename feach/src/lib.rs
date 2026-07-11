use minifb::{Key, Window, WindowOptions};
use togail::{COLS, Frame, Input, ROWS};

pub const WHITE: u32 = 0x00FF_FFFF;
pub const BLACK: u32 = 0x0000_0000;
pub const GHOST: u32 = 0x0055_5555;

pub fn board_to_pixels(board: &[[bool; COLS]; ROWS], scale: usize) -> Vec<u32> {
    let width = COLS * scale;
    let height = ROWS * scale;
    let mut pixels = vec![BLACK; width * height];

    for row in 0..ROWS {
        for col in 0..COLS {
            if board[row][col] {
                for dy in 0..scale {
                    for dx in 0..scale {
                        pixels[(row * scale + dy) * width + (col * scale + dx)] = WHITE;
                    }
                }
            }
        }
    }

    pixels
}

pub fn overlay_ghost(pixels: &mut [u32], ghost: &[(usize, usize); 4], scale: usize) {
    let width = COLS * scale;
    for &(col, row) in ghost {
        for dy in 0..scale {
            for dx in 0..scale {
                let idx = (row * scale + dy) * width + (col * scale + dx);
                if pixels[idx] == BLACK {
                    pixels[idx] = GHOST;
                }
            }
        }
    }
}

pub const SCALE: usize = 32;
pub const WIDTH: usize = COLS * SCALE;
pub const HEIGHT: usize = ROWS * SCALE;

pub fn should_quit(inputs: &[Input]) -> bool {
    inputs.iter().any(|i| matches!(i, Input::Quit))
}

pub fn parse_debug_level(args: &[String]) -> u32 {
    args.iter()
        .position(|a| a == "--level")
        .and_then(|i| args.get(i + 1))
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
}

const DAS_MS: u32 = 167;
const ARR_MS: u32 = 33;

const ALL_INPUTS: [Input; 8] = [
    Input::Left,
    Input::Right,
    Input::RotateCw,
    Input::RotateCcw,
    Input::SoftDrop,
    Input::HardDrop,
    Input::Pause,
    Input::Quit,
];

fn input_index(input: Input) -> usize {
    match input {
        Input::Left => 0,
        Input::Right => 1,
        Input::RotateCw => 2,
        Input::RotateCcw => 3,
        Input::SoftDrop => 4,
        Input::HardDrop => 5,
        Input::Pause => 6,
        Input::Quit => 7,
    }
}

pub struct InputHandler {
    held_ms: [u32; 8],
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHandler {
    pub fn new() -> Self {
        Self { held_ms: [0; 8] }
    }

    pub fn process(&mut self, active: &[Input], delta_ms: u32) -> Vec<Input> {
        let mut is_active = [false; 8];
        for &input in active {
            is_active[input_index(input)] = true;
        }

        let mut result = Vec::new();
        for (i, &input) in ALL_INPUTS.iter().enumerate() {
            if is_active[i] {
                let prev = self.held_ms[i];
                self.held_ms[i] = self.held_ms[i].saturating_add(delta_ms);
                let cur = self.held_ms[i];

                let fires = match input {
                    Input::HardDrop | Input::Pause | Input::Quit => prev == 0,
                    Input::Left | Input::Right | Input::RotateCw | Input::RotateCcw => {
                        prev == 0
                            || (prev < DAS_MS && cur >= DAS_MS)
                            || (prev >= DAS_MS && prev / ARR_MS < cur / ARR_MS)
                    }
                    Input::SoftDrop => true,
                };

                if fires {
                    result.push(input);
                }
            } else {
                self.held_ms[i] = 0;
            }
        }
        result
    }
}

pub fn run(mut frame_source: impl FnMut(&[Input], u32) -> Frame) {
    let mut window = Window::new("chechrish", WIDTH, HEIGHT, WindowOptions::default())
        .expect("failed to create window");

    window.set_target_fps(60);

    let mut input_handler = InputHandler::new();

    while window.is_open() {
        let raw: Vec<Input> = window
            .get_keys()
            .iter()
            .filter_map(|&k| map_key(k))
            .collect();

        if should_quit(&raw) {
            break;
        }

        let inputs = input_handler.process(&raw, 16);
        let frame = frame_source(&inputs, 16);
        let mut pixels = board_to_pixels(&frame.board, SCALE);
        if let Some(ghost) = &frame.ghost {
            overlay_ghost(&mut pixels, ghost, SCALE);
        }

        window
            .update_with_buffer(&pixels, WIDTH, HEIGHT)
            .expect("failed to update buffer");
    }
}

pub fn map_key(key: Key) -> Option<Input> {
    match key {
        Key::Left | Key::A => Some(Input::Left),
        Key::Right | Key::D => Some(Input::Right),
        Key::Up | Key::W | Key::Z => Some(Input::RotateCw),
        Key::X => Some(Input::RotateCcw),
        Key::Down | Key::S => Some(Input::SoftDrop),
        Key::Space => Some(Input::HardDrop),
        Key::P => Some(Input::Pause),
        Key::Escape | Key::Q => Some(Input::Quit),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn no_level_flag_defaults_to_zero() {
        assert_eq!(parse_debug_level(&args(&[])), 0);
    }

    #[test]
    fn level_flag_parses_value() {
        assert_eq!(parse_debug_level(&args(&["--level", "10"])), 10);
    }

    #[test]
    fn level_flag_with_missing_value_defaults_to_zero() {
        assert_eq!(parse_debug_level(&args(&["--level"])), 0);
    }

    #[test]
    fn level_flag_with_non_numeric_value_defaults_to_zero() {
        assert_eq!(parse_debug_level(&args(&["--level", "abc"])), 0);
    }

    fn empty_board() -> [[bool; COLS]; ROWS] {
        [[false; COLS]; ROWS]
    }

    fn full_board() -> [[bool; COLS]; ROWS] {
        [[true; COLS]; ROWS]
    }

    fn board_with_cell(col: usize, row: usize) -> [[bool; COLS]; ROWS] {
        let mut board = empty_board();
        board[row][col] = true;
        board
    }

    #[test]
    fn buffer_length_equals_cols_times_rows_times_scale_squared() {
        let scale = 4;
        let pixels = board_to_pixels(&empty_board(), scale);
        assert_eq!(pixels.len(), COLS * scale * ROWS * scale);
    }

    #[test]
    fn empty_board_produces_all_black() {
        let pixels = board_to_pixels(&empty_board(), 4);
        assert!(pixels.iter().all(|&p| p == BLACK));
    }

    #[test]
    fn full_board_produces_all_white() {
        let pixels = board_to_pixels(&full_board(), 4);
        assert!(pixels.iter().all(|&p| p == WHITE));
    }

    #[test]
    fn lit_cell_at_origin_fills_correct_block() {
        let scale = 4;
        let width = COLS * scale;
        let pixels = board_to_pixels(&board_with_cell(0, 0), scale);

        for dy in 0..scale {
            for dx in 0..scale {
                let idx = dy * width + dx;
                assert_eq!(pixels[idx], WHITE, "expected white at pixel ({dx},{dy})");
            }
        }
        // pixel just outside the block must be black
        assert_eq!(pixels[scale], BLACK);
        assert_eq!(pixels[scale * width], BLACK);
    }

    #[test]
    fn lit_cell_at_arbitrary_position_fills_correct_block() {
        let col = 3;
        let row = 5;
        let scale = 4;
        let width = COLS * scale;
        let pixels = board_to_pixels(&board_with_cell(col, row), scale);

        for dy in 0..scale {
            for dx in 0..scale {
                let idx = (row * scale + dy) * width + (col * scale + dx);
                assert_eq!(
                    pixels[idx], WHITE,
                    "expected white at pixel ({dx},{dy}) within cell ({col},{row})"
                );
            }
        }
    }

    #[test]
    fn unlit_cells_surrounding_a_lit_cell_are_black() {
        let scale = 4;
        let width = COLS * scale;
        let pixels = board_to_pixels(&board_with_cell(1, 1), scale);

        // row above the lit cell
        for x in 0..width {
            assert_eq!(pixels[x], BLACK, "row 0 pixel {x} should be black");
        }
        // column to the left of the lit cell within the same row band
        for dy in 0..scale {
            let idx = (scale + dy) * width; // col 0
            assert_eq!(pixels[idx], BLACK);
        }
    }

    #[test]
    fn scale_1_maps_each_cell_to_exactly_one_pixel() {
        let board = board_with_cell(2, 3);
        let pixels = board_to_pixels(&board, 1);
        assert_eq!(pixels.len(), COLS * ROWS);
        assert_eq!(pixels[3 * COLS + 2], WHITE);
        assert_eq!(pixels[3 * COLS + 1], BLACK);
        assert_eq!(pixels[2 * COLS + 2], BLACK);
    }

    // overlay_ghost tests

    fn ghost_at(col: usize, row: usize) -> [(usize, usize); 4] {
        [
            (col, row),
            (col + 1, row),
            (col, row + 1),
            (col + 1, row + 1),
        ]
    }

    #[test]
    fn ghost_cell_renders_as_ghost_colour() {
        let mut pixels = board_to_pixels(&empty_board(), 1);
        overlay_ghost(&mut pixels, &ghost_at(0, 0), 1);
        assert_eq!(pixels[0], GHOST);
    }

    #[test]
    fn ghost_does_not_overwrite_active_piece_cells() {
        let mut pixels = board_to_pixels(&board_with_cell(0, 0), 1);
        overlay_ghost(&mut pixels, &ghost_at(0, 0), 1);
        assert_eq!(pixels[0], WHITE);
    }

    #[test]
    fn ghost_does_not_affect_non_ghost_pixels() {
        let mut pixels = board_to_pixels(&empty_board(), 1);
        overlay_ghost(&mut pixels, &ghost_at(0, 0), 1);
        assert_eq!(pixels[5], BLACK);
    }

    #[test]
    fn ghost_renders_full_scaled_block() {
        let scale = 4;
        let width = COLS * scale;
        let mut pixels = board_to_pixels(&empty_board(), scale);
        let ghost = [(1usize, 1usize), (2, 1), (1, 2), (2, 2)];
        overlay_ghost(&mut pixels, &ghost, scale);
        for dy in 0..scale {
            for dx in 0..scale {
                let idx = (1 * scale + dy) * width + (1 * scale + dx);
                assert_eq!(pixels[idx], GHOST, "expected GHOST at dy={dy} dx={dx}");
            }
        }
    }

    #[test]
    fn ghost_overlapping_piece_leaves_piece_white_and_non_overlap_as_ghost() {
        let scale = 1;
        let mut board = empty_board();
        board[0][0] = true;
        let mut pixels = board_to_pixels(&board, scale);
        let ghost = [(0usize, 0usize), (1, 0), (0, 1), (1, 1)];
        overlay_ghost(&mut pixels, &ghost, scale);
        assert_eq!(pixels[0], WHITE);
        assert_eq!(pixels[1], GHOST);
        assert_eq!(pixels[COLS], GHOST);
    }

    // map_key tests

    #[test]
    fn left_arrow_maps_to_left() {
        assert!(matches!(map_key(Key::Left), Some(Input::Left)));
    }

    #[test]
    fn a_maps_to_left() {
        assert!(matches!(map_key(Key::A), Some(Input::Left)));
    }

    #[test]
    fn right_arrow_maps_to_right() {
        assert!(matches!(map_key(Key::Right), Some(Input::Right)));
    }

    #[test]
    fn d_maps_to_right() {
        assert!(matches!(map_key(Key::D), Some(Input::Right)));
    }

    #[test]
    fn up_arrow_maps_to_rotate_cw() {
        assert!(matches!(map_key(Key::Up), Some(Input::RotateCw)));
    }

    #[test]
    fn w_maps_to_rotate_cw() {
        assert!(matches!(map_key(Key::W), Some(Input::RotateCw)));
    }

    #[test]
    fn z_maps_to_rotate_cw() {
        assert!(matches!(map_key(Key::Z), Some(Input::RotateCw)));
    }

    #[test]
    fn x_maps_to_rotate_ccw() {
        assert!(matches!(map_key(Key::X), Some(Input::RotateCcw)));
    }

    #[test]
    fn down_arrow_maps_to_soft_drop() {
        assert!(matches!(map_key(Key::Down), Some(Input::SoftDrop)));
    }

    #[test]
    fn s_maps_to_soft_drop() {
        assert!(matches!(map_key(Key::S), Some(Input::SoftDrop)));
    }

    #[test]
    fn space_maps_to_hard_drop() {
        assert!(matches!(map_key(Key::Space), Some(Input::HardDrop)));
    }

    #[test]
    fn p_maps_to_pause() {
        assert!(matches!(map_key(Key::P), Some(Input::Pause)));
    }

    #[test]
    fn escape_maps_to_quit() {
        assert!(matches!(map_key(Key::Escape), Some(Input::Quit)));
    }

    #[test]
    fn q_maps_to_quit() {
        assert!(matches!(map_key(Key::Q), Some(Input::Quit)));
    }

    #[test]
    fn unmapped_key_returns_none() {
        assert!(map_key(Key::F1).is_none());
        assert!(map_key(Key::Tab).is_none());
        assert!(map_key(Key::NumPad0).is_none());
    }

    // dimension constant tests

    #[test]
    fn width_is_cols_times_scale() {
        assert_eq!(WIDTH, 320);
    }

    #[test]
    fn height_is_rows_times_scale() {
        assert_eq!(HEIGHT, 640);
    }

    // should_quit tests

    #[test]
    fn quit_input_signals_quit() {
        assert!(should_quit(&[Input::Quit]));
    }

    #[test]
    fn empty_inputs_do_not_quit() {
        assert!(!should_quit(&[]));
    }

    #[test]
    fn non_quit_inputs_do_not_quit() {
        assert!(!should_quit(&[Input::Left, Input::Right, Input::HardDrop]));
    }

    #[test]
    fn quit_among_other_inputs_signals_quit() {
        assert!(should_quit(&[Input::Left, Input::Quit, Input::Right]));
    }

    // InputHandler tests

    #[test]
    fn rotation_fires_on_initial_press() {
        let mut handler = InputHandler::new();
        let result = handler.process(&[Input::RotateCw], 16);
        assert!(result.contains(&Input::RotateCw));
    }

    #[test]
    fn rotation_does_not_fire_during_das_delay() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::RotateCw], 16);
        let result = handler.process(&[Input::RotateCw], 16);
        assert!(!result.contains(&Input::RotateCw));
    }

    #[test]
    fn rotation_fires_again_after_das_delay() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::RotateCw], 16);
        let result = handler.process(&[Input::RotateCw], DAS_MS);
        assert!(result.contains(&Input::RotateCw));
    }

    #[test]
    fn rotation_fires_again_after_release_and_repress() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::RotateCw], 16);
        handler.process(&[], 16);
        let result = handler.process(&[Input::RotateCw], 16);
        assert!(result.contains(&Input::RotateCw));
    }

    #[test]
    fn movement_fires_on_initial_press() {
        let mut handler = InputHandler::new();
        let result = handler.process(&[Input::Left], 16);
        assert!(result.contains(&Input::Left));
    }

    #[test]
    fn movement_does_not_fire_during_das_delay() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::Left], 16);
        let result = handler.process(&[Input::Left], 16);
        assert!(!result.contains(&Input::Left));
    }

    #[test]
    fn movement_fires_again_after_das_delay() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::Left], 16);
        let result = handler.process(&[Input::Left], DAS_MS);
        assert!(result.contains(&Input::Left));
    }

    #[test]
    fn movement_repeats_at_arr_rate_after_das() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::Left], 16);
        handler.process(&[Input::Left], DAS_MS);
        let result = handler.process(&[Input::Left], ARR_MS);
        assert!(result.contains(&Input::Left));
    }

    #[test]
    fn soft_drop_fires_every_frame_while_held() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::SoftDrop], 16);
        let result = handler.process(&[Input::SoftDrop], 16);
        assert!(result.contains(&Input::SoftDrop));
    }

    #[test]
    fn released_key_does_not_fire() {
        let mut handler = InputHandler::new();
        handler.process(&[Input::Left], 16);
        let result = handler.process(&[], 16);
        assert!(!result.contains(&Input::Left));
    }
}
