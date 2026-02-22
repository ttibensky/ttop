# UI Design

## Overview

ttop renders a full-screen interface using the terminal's alternate screen buffer. The display is divided into **boxed sections** stacked vertically, each representing a hardware subsystem. All metrics are displayed as **sparkline charts** with a 1-minute rolling history.

## Screen Layout

```
╭─ CPU ──────────────────────────────────────────────────────────────────────────╮
│                                                                                │
│   0 ▁▁▂▂▃▃▄▃▂▁▁▂▃▄▅▆▅▄▃▂▂▃▄▅▅▆▅▄▃▃▄▅▆▆▅▄▃▂▃▄▅▆▅▄▃▃▄▅▅▆▆▅▅▆▅▅  52% │
│   1 ▁▁▁▁▁▂▂▂▃▂▂▁▁▁▁▂▂▃▂▁▁▁▁▂▂▂▃▂▁▁▁▁▂▂▃▂▁▁▁▁▁▂▂▃▂▁▁▂▂▂▃▂▂▂▂  20% │
│   2 ▆▇▇█▇▇▆▇████▇▇█████▇▆▇▇████▇▇█████▇▆▇████▇▇████▇▆▇██████  93% │
│   3 ▃▄▅▅▆▅▅▄▃▃▄▅▆▆▅▅▄▃▃▄▅▅▆▆▅▄▃▃▄▅▆▆▅▅▄▃▃▄▅▅▆▅▅▄▅▅▆▅▅▆▅▅▆▅  61% │
│   4 ▁▁▁▁▂▁▁▁▁▁▁▂▁▁▁▁▁▁▁▂▂▁▁▁▁▁▁▂▁▁▁▁▁▁▂▂▁▁▁▁▁▁▁▂▁▁▁▁▁▁▂▂▁▂  15% │
│   5 ▂▂▃▃▃▂▂▂▃▃▃▄▃▃▂▂▃▃▃▂▂▃▃▃▄▃▃▂▂▂▃▃▃▂▂▃▃▃▄▃▃▂▂▂▃▃▃▃▃▃▃▃▃▃  30% │
│   6 ▅▆▆▇▇██▇▆▅▅▆▇▇██▇▆▅▆▆▇▇██▇▆▅▅▆▇██▇▇▆▅▆▇▇██▇▆▅▆▇██▇▇██  85% │
│   7 ▂▃▃▄▄▃▃▂▂▃▃▄▄▃▃▂▃▃▄▄▃▃▂▂▃▃▄▃▃▂▃▃▄▄▃▃▂▂▃▃▄▃▃▂▃▃▄▄▃▃▄▃▄  42% │
│   8 ▁▁▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁   5% │
│   9 ▄▅▅▆▆▇▆▆▅▄▅▅▆▇▇▆▆▅▄▅▆▆▇▇▆▅▅▆▆▇▆▆▅▄▅▅▆▇▇▆▆▅▅▆▆▇▇▆▅▆▇▆  78% │
│  10 ▂▃▃▃▂▂▃▃▃▃▃▂▂▂▃▃▃▂▂▃▃▃▃▃▂▂▂▃▃▃▂▂▃▃▃▃▃▂▂▂▃▃▃▂▃▃▃▃▃▃▃▃▃  35% │
│  11 ▃▄▅▅▆▅▅▄▃▄▅▅▆▆▅▅▄▃▄▅▅▆▅▅▄▃▄▅▅▆▆▅▄▃▄▅▅▆▅▅▄▃▄▅▅▆▆▅▅▆▅▆  68% │
│  12 ▇██████▇███████████▇████████▇██████████████████▇█████████ 100% │
│  13 ▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁▁▂▁▁▁▁▁▁▁▁  10% │
│  14 ▂▃▃▄▃▃▂▂▃▃▄▄▃▃▂▃▃▄▃▃▂▂▃▃▄▄▃▃▂▃▃▄▃▃▂▃▃▄▄▃▃▂▃▃▄▃▃▄▃▄▄▃  45% │
│  15 ▃▄▅▅▅▅▄▃▃▄▅▅▅▅▅▄▃▃▄▅▅▅▅▄▃▃▄▅▅▅▅▅▄▃▃▄▅▅▅▅▄▃▃▄▅▅▅▅▅▅▅▅  65% │
│                                                                                │
╰────────────────────────────────────────────────────────────────────────────────╯
╭─ Memory ───────────────────────────────────────────────────────────────────────╮
│                                                                                │
│  RAM ▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃  35% │
│  SWP ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁   6% │
│                                                                                │
╰────────────────────────────────────────────────────────────────────────────────╯
╭─ GPU ──────────────────────────────────────────────────────────────────────────╮
│                                                                                │
│  USE ▅▅▅▆▆▇▆▆▅▅▅▆▆▇▆▅▅▅▆▆▇▇▆▅▅▅▆▆▇▆▅▅▅▆▆▇▆▅▅▅▆▆▇▇▆▅▅▆▇▆▅▆▆▇▆▅▅  72% │
│  MEM ▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃  40% │
│                                                                                │
╰────────────────────────────────────────────────────────────────────────────────╯
                                                                  q: quit  ttop v0.1
```

## Sections

The screen is divided into independent boxed sections, stacked vertically:

1. **CPU** — one sparkline row per logical processor (thread), labeled 0–N
2. **Memory** — one row for RAM, one for swap
3. **GPU** — one row for utilization, one for memory (GPU name in section title)

Each section is enclosed in a box using Unicode box-drawing characters (`╭╮╰╯│─`) and has a labeled header. Sections are visually separated by the gap between boxes.

Future widgets (temperatures, disk I/O, network, etc.) are added as new boxed sections without disturbing existing ones.

## Sparkline Charts

### Character Set

Charts use the Unicode Block Elements to encode utilization as vertical height:

| Character | Utilization Range |
|-----------|-------------------|
| `▁` | 0–12% |
| `▂` | 13–25% |
| `▃` | 26–37% |
| `▄` | 38–50% |
| `▅` | 51–62% |
| `▆` | 63–75% |
| `▇` | 76–87% |
| `█` | 88–100% |

### History and Timing

- **Refresh interval:** 1 second
- **History depth:** 60 data points (1 minute)
- **Chart direction:** time flows left-to-right (oldest on the left, newest on the right)
- **Scrolling:** each tick, the chart shifts left by one position; the oldest value drops off and the newest is appended on the right

### Chart Width

The chart width is **dynamic** — it stretches to fill the available terminal width after accounting for fixed-width elements (core ID label, percentage, padding, borders). On wider terminals this means more history is visible (e.g., 120 columns of chart = 2 minutes of history). The data buffer stores as many data points as the current chart width requires.

### Startup Behavior

On first launch, the chart fills progressively from right to left:

- Second 1: all positions are empty except the rightmost (1 data point)
- Second 2: two rightmost positions filled
- ...
- After N seconds (where N = chart width): chart is fully populated

Empty (no-data) positions render as a dim `▁` character to maintain the visual baseline.

## Row Format

Each data row follows this structure:

```
{label}  {sparkline_chart}  {current_value}%
```

- **Label:** right-aligned, fixed width — `0`–`15` for CPU cores, `RAM`/`SWP` for memory, `USE`/`MEM` for GPU
- **Sparkline:** variable width, fills available space
- **Current value:** right-aligned 3-character percentage of the most recent data point, followed by `%`

## Color Scheme

### Utilization Colors (per-character)

Each character in the sparkline is **independently colored** based on its own value. This creates a natural heatmap effect where load transitions are visible as color gradients across the chart.

| Utilization | Color | ANSI Code |
|-------------|-------|-----------|
| 0–25% | Green | `\x1b[32m` (standard green) |
| 26–50% | Yellow | `\x1b[33m` (standard yellow) |
| 51–75% | Orange | `\x1b[38;5;208m` (256-color orange) |
| 76–100% | Red | `\x1b[31m` (standard red) |

The same color thresholds apply to temperatures when they are added (with degree-based ranges instead of percentages).

### Current Value Percentage

The percentage number on the right side of each row is colored to match the **current (most recent)** data point's color.

### Static Elements

| Element | Color | ANSI Code |
|---------|-------|-----------|
| Box borders (`╭╮╰╯│─`) | Dim gray | `\x1b[90m` |
| Section labels (`CPU`, `Memory`, `GPU`) | Bold cyan | `\x1b[1;36m` |
| Row labels (core IDs, `RAM`, `SWP`, etc.) | White | `\x1b[37m` |
| Brackets and separators | Dim gray | `\x1b[90m` |
| Empty chart positions (no data yet) | Dim gray | `\x1b[38;5;240m` |
| Status bar (`q: quit`, version) | Dim gray | `\x1b[90m` |

## Terminal Handling

- **Full-screen mode:** uses the alternate screen buffer (like htop); the original terminal content is restored on exit
- **Cursor:** hidden during operation
- **Exit:** `q` key or `Ctrl+C` triggers a clean shutdown (restore screen, show cursor, disable raw mode)
- **Resize:** the layout dynamically adapts when the terminal is resized — chart widths recalculate, data buffers grow or shrink accordingly

## Status Bar

A minimal status bar is rendered at the bottom-right of the screen:

```
q: quit  ttop v0.1
```

Rendered in dim gray, it stays out of the way while providing essential information.
