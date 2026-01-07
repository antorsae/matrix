# Matrix Rain CLI

A Matrix movie-style terminal animation written in Rust.

![Matrix Rain Demo](demo.gif)

## Features

- Authentic Matrix aesthetic with Japanese katakana + ASCII characters
- 256-color gradient (white head → bright/medium/dim green trail)
- 3 speed tiers for depth perception (foreground/background effect)
- Smooth 30 FPS animation with delta-time movement
- Differential rendering for performance
- Cross-platform support (Windows, macOS, Linux)

## Installation

### From source

```bash
cargo build --release
./target/release/matrix
```

### Run directly

```bash
cargo run --release
```

## Usage

```bash
./matrix
```

Press `q`, `Esc`, or `Ctrl+C` to exit.

## Requirements

- A terminal with color support (256-color or true color recommended)
- Minimum terminal size: 20x10

---

## How This Was Developed

This project demonstrates **AI-assisted development** using Claude Code with the [dual-agent](https://github.com/antorsae/dual-agent) collaboration pattern.

### Step 1: The Original Spec

The entire project started with this one-line spec in `SPEC.md`:

```
Write a python program CLI that outputs to the terminal a MATRIX MOVIE LIKE animation (green stuff)
```

### Step 2: AI Interview Process

Instead of jumping straight into implementation, Claude was prompted to conduct a detailed requirements interview:

```
read this @SPEC.md and interview me in detail using the AskUserQuestionTool
about literally anything: technical implementation, UI & UX, concerns, tradeoffs, etc.
but make sure the questions are not obvious
be very in-depth and continue interviewing me continually until it's complete,
then write the spec to the file
```

This resulted in **6 rounds of detailed questions** covering:

| Topic | Questions Asked |
|-------|-----------------|
| Character set | Katakana + ASCII vs ASCII-only vs configurable |
| Rain behavior | Independent columns vs speed tiers vs density waves |
| Rendering engine | Raw ANSI vs curses vs Rich/blessed |
| Trail effects | 2-level vs 4-level gradient, variable trail length |
| Spawn logic | Immediate respawn vs delayed vs pool-based |
| Character mutation | Static vs periodic vs head-only mutation |
| FPS target | 15-20 vs 30 vs adaptive |
| Terminal handling | Resize behavior, TTY validation, min size |
| CLI interface | Minimal vs essential flags vs full control |
| Exit behavior | Instant vs fade-out vs clear+message |
| Color depth | 16-color vs 256-color vs true color |
| Column density | Fixed percentage vs absolute count vs width-scaled |
| Error handling | Curses init failure, terminal too small |

The interview transformed a vague one-liner into a [detailed 200-line technical specification](SPEC.md).

### Step 3: Implementation & Rust Rewrite

The project was initially implemented in Python, then rewritten in Rust for:
- Better performance (native binary)
- Cross-platform support via crossterm
- No runtime dependencies

---

## Technical Details

See [SPEC.md](SPEC.md) for the complete technical specification generated through the interview process.

### Architecture

```
src/main.rs
├── Constants (CHAR_SET, SPEED_TIERS, colors, timing)
├── struct Column (rain column state + movement)
├── struct MatrixRain (main controller + rendering)
└── main() with terminal setup/cleanup
```

### Key Algorithms

- **Movement**: `y_head += speed * delta_time` (frame-rate independent)
- **Differential rendering**: Only update cells that changed between frames
- **Color gradient**: Position-based color assignment (head=white, trail=green gradient)

### Dependencies

- `crossterm` - Cross-platform terminal manipulation
- `rand` - Random number generation

## License

MIT
