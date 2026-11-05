# 2048 Contract

## Purpose

`2048` is an interactive terminal game. It renders one 4x4 board, accepts keyboard moves, persists local high scores, and adapts its geometry to terminal resize events.

## Command surface

```text
2048 [OPTIONS]
```

| Option | Accepted values | Default | Contract |
|---|---|---|---|
| `--layout` | `auto`, `compact`, `balanced`, `large`, `manual` | `auto` | Selects geometry policy. |
| `--tile-width` | integer greater than or equal to 5 | none | Valid only with `--layout manual`. |
| `--tile-height` | integer greater than or equal to 1 | none | Valid only with `--layout manual`. |
| `-h`, `--help` | none | none | Prints help without requiring a TTY. |
| `-V`, `--version` | none | none | Prints the package version without requiring a TTY. |

Manual mode requires both dimensions. Unknown options, malformed dimensions, partial manual dimensions, and manual dimensions outside manual mode exit with status 2 and a diagnostic on stderr.

## Rendering contract

- `auto` derives tile width and height independently from the current terminal dimensions.
- The board never draws beyond the terminal geometry reported for the current frame.
- A terminal that cannot hold the selected layout receives an explicit required-versus-actual size message.
- Resize events trigger a complete redraw using the new geometry.
- Tile labels use three-row block numerals when both dimensions permit them; otherwise they use a centered plain number.
- Preset and manual layouts remain centered instead of stretching.

## Input contract

- Arrow keys and `WASD` move the board.
- `H` opens the high-score view.
- `R` starts a new board immediately.
- `Q` and `Esc` leave the game.
- A new tile appears only after a move changes the board.

## State contract

- The ten highest scores are persisted as JSON.
- The canonical path is `$XDG_DATA_HOME/2048/high-scores.json`, falling back to `$HOME/.local/share/2048/high-scores.json` through the platform data-directory API.
- Legacy `$HOME/.2048_high_scores.json` data is loaded and copied to the canonical path when no canonical file exists.
- Corrupt or unreadable score content does not prevent the game from starting; it yields an empty in-memory score table.

## Terminal lifecycle

- Interactive play requires stdout to be attached to a terminal.
- Help and version output work without a terminal.
- Normal and error returns restore raw mode and leave the alternate screen.

## Error model

| Condition | Behavior |
|---|---|
| Invalid CLI input | Diagnostic and help on stderr, exit 2. |
| Non-interactive gameplay | Diagnostic on stderr, no game session. |
| Selected layout does not fit | In-TUI size diagnostic; resize or quit remains possible. |
| Score file absent or invalid | Start with an empty score table. |
| Terminal backend failure | Restore terminal state when possible and return the I/O error. |
