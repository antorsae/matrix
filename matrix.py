#!/usr/bin/env python3
"""Matrix-style terminal rain animation."""

import curses
import locale
import random
import signal
import sys
import time
from dataclasses import dataclass, field

# Character sets
KATAKANA = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲン"
ASCII_CHARS = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz!@#$%^&*()_+-=[]{}|;:',.<>?/"
CHAR_SET = list(KATAKANA + ASCII_CHARS)

# Speed tiers (cells per second) - 1x/2x/3x for depth perception
SPEED_TIERS = [8.0, 16.0, 24.0]

# Visual parameters
TRAIL_LENGTH_RANGE = (8, 25)
COLUMN_DENSITY = 0.65  # 60-70%
MUTATION_RATE = 0.10  # 10% per frame

# Timing
TARGET_FPS = 30
FRAME_TIME = 1.0 / TARGET_FPS

# Terminal requirements
MIN_WIDTH = 20
MIN_HEIGHT = 10

# Color pair indices
COLOR_HEAD = 1
COLOR_BRIGHT = 2
COLOR_MEDIUM = 3
COLOR_DIM = 4


@dataclass
class Column:
    """Represents a single falling rain column."""

    x: int
    screen_height: int
    y_head: float = 0.0
    speed_tier: int = field(default_factory=lambda: random.randint(0, 2))
    trail_length: int = field(default_factory=lambda: random.randint(*TRAIL_LENGTH_RANGE))
    characters: list[str] = field(default_factory=list)
    active: bool = True

    def __post_init__(self):
        self.speed = SPEED_TIERS[self.speed_tier]
        if not self.characters:
            self.characters = [random.choice(CHAR_SET) for _ in range(self.trail_length)]

    def update(self, delta_time: float) -> None:
        """Move column down by delta_time * speed."""
        self.y_head += self.speed * delta_time

        # Check if fully off screen (head + trail length past bottom)
        if self.y_head - self.trail_length > self.screen_height:
            self.active = False

    def mutate(self) -> None:
        """Randomly mutate characters based on MUTATION_RATE."""
        for i in range(len(self.characters)):
            if random.random() < MUTATION_RATE:
                self.characters[i] = random.choice(CHAR_SET)

    def get_visible_cells(self) -> list[tuple[int, int, str, int]]:
        """Return list of (x, y, char, color_pair) for visible cells."""
        cells = []
        head_y = int(self.y_head)

        for i in range(self.trail_length):
            y = head_y - i
            if 0 <= y < self.screen_height:
                char = self.characters[i % len(self.characters)]
                color = self._get_color_for_position(i)
                cells.append((self.x, y, char, color))

        return cells

    def _get_color_for_position(self, pos: int) -> int:
        """Determine color pair based on position in trail."""
        if pos == 0:
            return COLOR_HEAD  # White head
        ratio = pos / self.trail_length
        if ratio < 0.33:
            return COLOR_BRIGHT  # Bright green
        elif ratio < 0.66:
            return COLOR_MEDIUM  # Medium green
        else:
            return COLOR_DIM  # Dim green


class MatrixRain:
    """Main application controller."""

    def __init__(self, stdscr):
        self.stdscr = stdscr
        self.height = 0
        self.width = 0
        self.columns: list[Column] = []
        self.column_slots: set[int] = set()  # Active x positions
        self.running = True
        self.prev_frame: dict[tuple[int, int], tuple[str, int]] = {}

    def setup(self) -> None:
        """Initialize curses settings and validate terminal."""
        # Initialize locale for proper Unicode rendering
        locale.setlocale(locale.LC_ALL, "")

        # Hide cursor (some terminals don't support this)
        try:
            curses.curs_set(0)
        except curses.error:
            pass

        # Non-blocking input
        self.stdscr.nodelay(True)
        self.stdscr.timeout(0)

        # Get terminal size
        self.height, self.width = self.stdscr.getmaxyx()

        # Validate minimum size
        if self.width < MIN_WIDTH or self.height < MIN_HEIGHT:
            raise RuntimeError(
                f"Terminal too small: {self.width}x{self.height}. "
                f"Minimum size: {MIN_WIDTH}x{MIN_HEIGHT}."
            )

        # Setup colors
        self._setup_colors()

        # Clear screen and set background
        self.stdscr.bkgd(" ", curses.color_pair(0))
        self.stdscr.clear()

        # Initialize columns at full density
        self._spawn_initial_columns()

    def _setup_colors(self) -> None:
        """Initialize color pairs for gradient with fallback for limited terminals."""
        curses.start_color()
        curses.use_default_colors()

        if not curses.has_colors():
            raise RuntimeError("Terminal does not support colors.")

        if curses.COLORS >= 256:
            # 256-color palette indices:
            # 255 = bright white (head)
            # 46  = bright green (#00ff00)
            # 40  = medium green (#00d700)
            # 34  = dim green (#00af00)
            curses.init_pair(COLOR_HEAD, 255, -1)
            curses.init_pair(COLOR_BRIGHT, 46, -1)
            curses.init_pair(COLOR_MEDIUM, 40, -1)
            curses.init_pair(COLOR_DIM, 34, -1)
        else:
            # Fallback to 8-color mode
            curses.init_pair(COLOR_HEAD, curses.COLOR_WHITE, -1)
            curses.init_pair(COLOR_BRIGHT, curses.COLOR_GREEN, -1)
            curses.init_pair(COLOR_MEDIUM, curses.COLOR_GREEN, -1)
            curses.init_pair(COLOR_DIM, curses.COLOR_GREEN, -1)

    def _spawn_initial_columns(self) -> None:
        """Spawn columns to achieve target density immediately."""
        target_count = int(self.width * COLUMN_DENSITY)
        available_slots = list(range(self.width))
        random.shuffle(available_slots)

        for x in available_slots[:target_count]:
            col = Column(x=x, screen_height=self.height)
            # Randomize starting position for varied entry
            col.y_head = random.uniform(-col.trail_length, self.height)
            self.columns.append(col)
            self.column_slots.add(x)

    def _spawn_new_column(self) -> None:
        """Spawn a new column at random available position."""
        available = [x for x in range(self.width) if x not in self.column_slots]
        if available:
            x = random.choice(available)
            col = Column(x=x, screen_height=self.height)
            col.y_head = 0.0  # Start from top
            self.columns.append(col)
            self.column_slots.add(x)

    def update(self, delta_time: float) -> None:
        """Update all columns and manage spawning."""
        # Update existing columns
        for col in self.columns:
            col.update(delta_time)
            col.mutate()

        # Remove inactive columns
        inactive = [c for c in self.columns if not c.active]
        for col in inactive:
            self.column_slots.discard(col.x)
        self.columns = [c for c in self.columns if c.active]

        # Spawn replacements to maintain density
        target_count = int(self.width * COLUMN_DENSITY)
        while len(self.column_slots) < target_count:
            self._spawn_new_column()

    def render(self) -> None:
        """Render frame with differential updates."""
        # Build current frame state
        current_frame: dict[tuple[int, int], tuple[str, int]] = {}

        for col in self.columns:
            for x, y, char, color in col.get_visible_cells():
                # Avoid bottom-right corner (curses quirk)
                if 0 <= x < self.width and 0 <= y < self.height:
                    if x == self.width - 1 and y == self.height - 1:
                        continue
                    current_frame[(x, y)] = (char, color)

        # Clear cells that were drawn last frame but not this frame
        for pos in self.prev_frame:
            if pos not in current_frame:
                x, y = pos
                try:
                    self.stdscr.addch(y, x, " ")
                except curses.error:
                    pass

        # Draw new/changed cells
        for pos, (char, color) in current_frame.items():
            if pos not in self.prev_frame or self.prev_frame[pos] != (char, color):
                x, y = pos
                try:
                    self.stdscr.addch(y, x, char, curses.color_pair(color))
                except curses.error:
                    pass

        self.prev_frame = current_frame
        self.stdscr.refresh()

    def run(self) -> None:
        """Main loop with frame pacing."""
        self.setup()
        last_time = time.monotonic()

        while self.running:
            current_time = time.monotonic()
            delta_time = current_time - last_time

            # Check for quit key (q)
            try:
                key = self.stdscr.getch()
                if key == ord("q"):
                    self.running = False
            except curses.error:
                pass

            # Update simulation
            self.update(delta_time)

            # Render
            self.render()

            # Frame pacing
            elapsed = time.monotonic() - current_time
            sleep_time = FRAME_TIME - elapsed
            if sleep_time > 0:
                time.sleep(sleep_time)

            last_time = current_time


def main(stdscr) -> None:
    """Curses wrapper entry point."""
    app = MatrixRain(stdscr)
    app.run()


def signal_handler(signum, frame) -> None:
    """Handle Ctrl+C for clean exit."""
    sys.exit(0)


if __name__ == "__main__":
    # Register signal handlers before curses init
    signal.signal(signal.SIGINT, signal_handler)
    signal.signal(signal.SIGTERM, signal_handler)

    try:
        # Check TTY requirement
        if not sys.stdout.isatty():
            print("Error: Requires TTY.", file=sys.stderr)
            sys.exit(1)

        # Run with curses wrapper (handles init/cleanup)
        curses.wrapper(main)
    except curses.error as e:
        print(
            f"Error: Cannot initialize terminal. "
            f"Ensure TERM is set and you're running in a supported terminal.\n"
            f"Details: {e}",
            file=sys.stderr,
        )
        sys.exit(1)
    except RuntimeError as e:
        print(f"Error: {e}", file=sys.stderr)
        sys.exit(1)
