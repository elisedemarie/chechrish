#![no_std]
extern crate alloc;

mod board;
mod game;
mod input;
mod random;
mod shape;

pub use input::Input;

pub use crate::game::Game;

pub const COLS: usize = 10;
pub const ROWS: usize = 20;
pub const GRAVITY_TICK: u32 = 1000;

pub struct Frame {
    pub board: [[bool; COLS]; ROWS],
    pub score: u32,
    pub level: u32,
    pub ghost: Option<[(usize, usize); 4]>,
}
