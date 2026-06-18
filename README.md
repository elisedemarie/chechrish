# chechrish

A Tetris clone in Rust built with `no_std`.

## Crates

### togail

The game engine. `no_std` + `alloc` — no platform assumptions, fully portable. Pure logic: input in, new state out. Exposes a 2D bit grid, score, and level as plain data. Knows nothing about rendering.

### feach

The UI. Opens a native window using `minifb` and maps the engine's bit grid to a pixel buffer. Handles keyboard input and passes `Input` events to the engine. Targets 60fps. Handles DAS (Delayed Auto Shift) and ARR (Auto Repeat Rate) for smooth movement.

## Running

```
cargo run -p feach
```

## Architecture

The two crates share a clean interface: feach only ever sees a `[[bool; 10]; 20]` board, a score, a level, and an `Input` enum. togail never knows feach exists. This makes the engine trivially portable — a future platform backend just needs to produce inputs and consume a grid.
