# Matrix Rain CLI - Technical Specification

A Python CLI tool that renders a Matrix movie-style falling character animation in the terminal.

---

## Overview

The program displays cascading columns of characters (katakana + ASCII) falling down the terminal screen with varying speeds, creating a depth illusion reminiscent of the Matrix film's iconic "digital rain" effect.

---

## Visual Design

### Character Set
- **Mixed katakana and ASCII**: Half-width Japanese katakana characters combined with ASCII digits, symbols, and letters
- Provides authentic Matrix aesthetic while maintaining visual variety

### Color Palette
- **256-color mode** for smooth gradients
- Primary palette: shades of green
- **Head character**: Bright white (high contrast, stands out from trail)
- **Trail gradient**: 4-level fade
  1. Head: bright white
  2. Recent trail: bright green
  3. Mid trail: medium green
  4. Fading trail: dim green
  5. Gone (cleared)

### Column Behavior

#### Speed Tiers (Depth Simulation)
3 discrete speed tiers to create foreground/background perception:
- **Tier 1 (slow)**: 1x base speed — appears as "background"
- **Tier 2 (medium)**: 2x base speed — mid-layer
- **Tier 3 (fast)**: 3x base speed — appears as "foreground"

Each column is randomly assigned a tier at spawn.

#### Density
- **Fixed percentage**: ~60-70% of terminal columns active at any time
- Columns may spawn at **any position** (adjacent columns allowed)

#### Character Mutation
- **Periodic mutation**: Characters randomly change while visible
- **Mutation rate**: ~10% of visible characters change per frame
- Creates subtle "shimmer" effect without being too chaotic

#### Spawn & Lifecycle
- **Immediate random respawn**: When a column completes (fully scrolled off), a new column spawns instantly at a random column position
- No delay between column death and new column spawn

---

## Technical Implementation

### Rendering Engine
- **curses/ncurses** (Python `curses` module)
- Standard library on Unix; Windows requires `windows-curses` package

### Frame Rate
- **Target: 30 FPS**
- Use frame timing to maintain consistent animation speed regardless of rendering time

### Terminal Handling

#### Resize Behavior
- **Ignore resize events**: Use terminal dimensions captured at startup
- Simplifies implementation; user can restart if resize is needed

#### Minimum Terminal Size
- **Minimum width**: 20 columns
- **Minimum height**: 10 rows
- **Behavior**: Refuse to run with clean error message if terminal is too small

#### TTY Validation
- **Require TTY**: Exit with error if stdout is not a terminal
- Prevents broken output when accidentally piped

---

## CLI Interface

### Arguments
**None** — Minimal interface, just run the program.

```
$ python matrix.py
```

No flags, no configuration. Simplicity over customization.

### Exit Behavior
- **Ctrl+C (SIGINT)**: Instant termination
- Restore terminal state (cursor visibility, colors, alternate screen buffer)
- No fade-out animation, no goodbye message
- Clean, immediate exit

### Error Handling

#### curses Initialization Failure
- Print human-readable error message explaining the issue
- Exit with code 1
- Example: `"Error: Cannot initialize terminal. Ensure TERM is set and you're running in a supported terminal."`

#### Terminal Too Small
- Print error: `"Error: Terminal too small. Minimum size: 20x10."`
- Exit with code 1

---

## Implementation Details

### Data Structures

```
Column:
  - x_position: int
  - y_head: float (allows sub-cell positioning for smooth movement)
  - speed_tier: int (1-3)
  - trail_length: int
  - characters: list[str] (current visible characters)

Screen State:
  - columns: list[Column]
  - width: int
  - height: int
```

### Main Loop Pseudocode

```
1. Initialize curses, hide cursor, set up colors
2. Get terminal dimensions, validate minimum size
3. Spawn initial columns (60-70% density)
4. Loop:
   a. Calculate frame delta time
   b. For each column:
      - Advance head position based on speed tier
      - Apply character mutation (~10% chance per visible char)
      - If head is off-screen, respawn at random x with random tier
   c. Render frame:
      - Clear or overwrite changed cells only (optimization)
      - Draw each column with gradient colors
   d. Sleep to maintain 30 FPS
   e. Check for interrupt signal
5. On exit: restore terminal state
```

### Character Pool

```python
KATAKANA = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン"
ASCII = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=[]{}|;:',.<>?/"
CHARSET = KATAKANA + ASCII
```

### 256-Color Green Palette

```python
# Example color indices (approximate)
HEAD_COLOR = 231        # Bright white
BRIGHT_GREEN = 46       # #00ff00
MEDIUM_GREEN = 34       # #00af00
DIM_GREEN = 22          # #005f00
```

---

## Performance Considerations

- **Differential rendering**: Only update cells that changed (curses handles this efficiently)
- **Avoid per-frame allocations**: Reuse column objects, pre-allocate character buffers
- **Frame pacing**: Use monotonic clock for accurate timing, sleep for remainder of frame budget

---

## Dependencies

- Python 3.7+
- `curses` (stdlib on Unix)
- `windows-curses` (pip install, Windows only)

---

## Summary Table

| Aspect | Decision |
|--------|----------|
| Character set | Katakana + ASCII |
| Color depth | 256-color |
| Head color | Bright white |
| Trail gradient | 4 levels |
| Speed model | 3 tiers (1x/2x/3x) |
| Column density | 60-70% fixed |
| Column spacing | Any (adjacent allowed) |
| Char mutation | 10% per frame |
| Spawn behavior | Immediate random |
| Target FPS | 30 |
| Resize handling | Ignore (use initial) |
| CLI args | None |
| Exit | Instant on Ctrl+C |
| TTY required | Yes |
| Min terminal | 20×10 |
| Startup | Instant full density |
