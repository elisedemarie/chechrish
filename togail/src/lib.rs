#![no_std]
extern crate alloc;

mod shape;
mod board;
mod game;

pub const COLS: usize = 10;
pub const ROWS: usize = 20;

pub enum Input {
    Left,
    Right,
    RotateCw,
    RotateCcw,
    SoftDrop,
    HardDrop,
    Pause,
    Quit,
}

pub struct Frame {
    pub board: [[bool; COLS]; ROWS],
    pub score: u32,
    pub level: u32,
}
