# Matrix Rain CLI

A Matrix movie-style terminal animation written in Python.

```
    ア  7    ム    )   ヨ  $      コ   ワ        ネ
    シ  M    ヌ    L   3   @      テ   ヲ        ホ
   ウ  セ  P   ヘ   ネ  /   タ   =   ト   ン        ミ
   エ  ソ  Q   ホ   ノ  [   チ   +   ナ   0         ム
  オ  タ  R   マ   ハ  ]   ツ   _   ニ   1          メ
 カ  チ  S   ミ   ヒ  {   テ   <   ヌ   2           モ
 キ  ツ  T   ム   フ  }   ト   >   ネ   3            ヤ
ク  テ  U   メ   ヘ  |   ナ   ?   ノ   4             ユ
```

> Run `python matrix.py` to see the full animated effect!

## Features

- Authentic Matrix aesthetic with Japanese katakana + ASCII characters
- 256-color gradient (white head → bright/medium/dim green trail)
- 3 speed tiers for depth perception (foreground/background effect)
- Smooth 30 FPS animation with delta-time movement
- Differential rendering for performance
- Graceful fallback to 8-color terminals

## Usage

```bash
python matrix.py
```

Press `q` or `Ctrl+C` to exit.

## Requirements

- Python 3.7+
- A terminal with color support (256-color recommended)
- Unix/Linux/macOS (Windows requires `windows-curses`)

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

### Step 3: Dual-Agent Implementation

Using [dual-agent](https://github.com/antorsae/dual-agent), two AI agents collaborated:

#### Claude (Implementation Agent)
- Designed the architecture (Column dataclass + MatrixRain controller)
- Wrote the complete implementation (~300 lines)
- Implemented differential rendering algorithm
- Set up curses with 256-color support
- Added frame pacing and delta-time movement

#### Codex (Review Agent)
Claude delegated code review to Codex, which identified:

| Severity | Issue | Fix Applied |
|----------|-------|-------------|
| **High** | 256-color indices may fail on non-256-color terminals | Added `curses.COLORS` check with 8-color fallback |
| **Medium** | Need locale init for Katakana rendering | Added `locale.setlocale(locale.LC_ALL, "")` |
| **Medium** | Should guard color setup | Added `curses.has_colors()` check |
| **Low** | `curs_set(0)` may fail on some terminals | Wrapped in try/except |
| ✓ Pass | Dataclass mutable defaults | Correctly uses `default_factory` |

### The Result

From a one-line spec to a production-quality terminal animation with:
- Proper error handling
- Cross-terminal compatibility
- Performance optimizations
- Clean architecture

All through AI-assisted development with human guidance on design decisions.

---

## Technical Details

See [SPEC.md](SPEC.md) for the complete technical specification generated through the interview process.

### Architecture

```
matrix.py
├── Constants (CHAR_SET, SPEED_TIERS, colors, timing)
├── @dataclass Column (rain column state + movement)
├── class MatrixRain (main controller + rendering)
└── main() with signal handling
```

### Key Algorithms

- **Movement**: `y_head += speed * delta_time` (frame-rate independent)
- **Differential rendering**: Only update cells that changed between frames
- **Color gradient**: Position-based color assignment (head=white, trail=green gradient)

## License

MIT
