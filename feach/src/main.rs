use feach::run;
use togail::{COLS, Frame, ROWS};

fn checkerboard() -> [[bool; COLS]; ROWS] {
    let mut board = [[false; COLS]; ROWS];
    for (row, cells) in board.iter_mut().enumerate() {
        for (col, cell) in cells.iter_mut().enumerate() {
            *cell = (row + col) % 2 == 0;
        }
    }
    board
}

fn main() {
    let board = checkerboard();
    run(|_inputs| Frame {
        board,
        score: 0,
        level: 1,
    });
}
