#[derive(Copy, Clone, PartialEq, Eq, Debug)]
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
