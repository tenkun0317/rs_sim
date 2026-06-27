# RS Sim

A Minecraft redstone simulator written in Rust using [macroquad](https://github.com/not-fl3/macroquad).

## Features

- Place and simulate redstone components: wire, torch, repeater, comparator, lever, button, redstone block, lamp, target block, barrel, and solid blocks
- Real-time and step-by-step simulation modes
- Selection, copy/cut/paste areas
- Undo/redo (Ctrl+Z / Ctrl+Y)
- Save/load projects as JSON files
- Pan and zoom camera

## Controls

| Key | Action |
|---|---|
| Left click + drag | Place blocks / interact |
| Right click + drag | Select area |
| Ctrl+Z / Ctrl+Y | Undo / Redo |
| Ctrl+X / Ctrl+C / Ctrl+V | Cut / Copy / Paste |
| Ctrl+S / Ctrl+Shift+S | Save / Save As |
| Ctrl+R | Load |
| Space | Toggle simulation (Off / Timed / Instant) |
| Enter | Step simulation once |
| +/- | Adjust simulation speed |
| R | Clear world |
| C | Center camera |
| Tab / 1-0 | Cycle / Select block type |
| WASD / Arrow keys | Pan |
| Scroll wheel | Zoom |
| DEL | Delete selected area |
| ESC | Cancel selection / paste mode |

## Build & Run

```sh
cargo run --release
```

## License

This project is dedicated to the public domain under the Unlicense.
