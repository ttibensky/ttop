# UI Design

## Overview

ttop renders a full-screen interface using the terminal's alternate screen buffer. The display is divided into **boxed sections** stacked vertically, each representing a hardware subsystem. All metrics are displayed as **sparkline charts** with a 1-minute rolling history.

## Screen Layout

The CPU section uses a **side-by-side layout**: utilization sparklines on the left half, temperature sparklines on the right half, separated by a vertical `│` border.

```
╭─ CPU ───────────────────────────────────────────────────────────────────────────────────╮
│                                     │                                                   │
│ #0  ▁▁▂▂▃▃▄▃▂▁▁▂▃▄▅▆▅▄▃▂▂▃▄  52% │ Tctl ▁▁▁▁▂▂▂▂▂▂▃▃▃▃▃▃▃▃▃▃▃  46°C (115°F)       │
│ #1  ▁▁▁▁▁▂▂▂▃▂▂▁▁▁▁▂▂▃▂▁▁▁▁  20% │                                                   │
│ #2  ▆▇▇█▇▇▆▇████▇▇█████▇▆▇▇  93% │                                                   │
│ #3  ▃▄▅▅▆▅▅▄▃▃▄▅▆▆▅▅▄▃▃▄▅▅▆  61% │                                                   │
│ ...                                │                                                   │
│ #15 ▃▄▅▅▅▅▄▃▃▄▅▅▅▅▅▄▃▃▄▅▅▅▅  65% │                                                   │
│                                     │                                                   │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
╭─ Memory ────────────────────────────────────────────────────────────────────────────────╮
│                                                                                         │
│  RAM ▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃   35% │
│  SWP ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁    6% │
│                                                                                         │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
╭─ GPU ───────────────────────────────────────────────────────────────────────────────────╮
│                                                                                         │
│  USE ▅▅▅▆▆▇▆▆▅▅▅▆▆▇▆▅▅▅▆▆▇▇▆▅▅▅▆▆▇▆▅▅▅▆▆▇▆▅▅▅▆▆▇▇▆▅▅▆▇▆▅▆▆▇▆▅▅▆▆▇▆▅▅▅▆▆▇   72% │
│  MEM ▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃▃   40% │
│                                                                                         │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
                                                                        q: quit  ttop v0.1
```

**When no temperature sensors are found** (VMs, containers, etc.), the right half shows:

```
│  N/A ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  N/A°C (N/A°F)  │
```

## Sections

The screen is divided into independent boxed sections, stacked vertically:

1. **CPU** — split into two halves:
   - **Left half:** one utilization sparkline row per logical processor (thread), labeled 0–N, with current percentage
   - **Right half:** one temperature sparkline row per sensor (e.g., `Tctl` for AMD, `Core 0`–`Core N` for Intel), with dual °C/°F display
   - A vertical `│` separator divides the halves
   - Temperature rows are **top-aligned**: if there are fewer sensors than CPU cores, remaining right-half rows are blank
   - If no sensors found: a single `N/A°C (N/A°F)` row with dim styling
2. **Memory** (future) — one row for RAM, one for swap
3. **GPU** (future) — one row for utilization, one for memory (GPU name in section title)

Each section is enclosed in a box using Unicode box-drawing characters (`╭╮╰╯│─`) and has a labeled header. Sections are visually separated by the gap between boxes.

Future widgets (disk I/O, network, etc.) are added as new boxed sections without disturbing existing ones.

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

### Utilization Row (left half)

```
│ {label} {sparkline_chart} {NNN}% │
```

- **Label:** left-aligned, fixed width — `#0`–`#15` for CPU cores (with trailing space padding), `RAM`/`SWP` for memory, `USE`/`MEM` for GPU
- **Sparkline:** variable width, fills available left-half space
- **Current value:** right-aligned 3-character percentage

### Temperature Row (right half)

```
 {label} {sparkline_chart} {NNN}°C ({NNN}°F) │
```

- **Label:** right-aligned, fixed width — sensor label (e.g., `Tctl`, `Core 0`)
- **Sparkline:** variable width, fills available right-half space
- **Current value:** dual Celsius/Fahrenheit display, or `N/A°C (N/A°F)` if unavailable

## Color Scheme

### Utilization Colors (per-character)

Each character in the sparkline is **independently colored** based on its own value. This creates a natural heatmap effect where load transitions are visible as color gradients across the chart.

| Utilization | Color | ANSI Code |
|-------------|-------|-----------|
| 0–25% | Green | `\x1b[32m` (standard green) |
| 26–50% | Yellow | `\x1b[33m` (standard yellow) |
| 51–75% | Orange | `\x1b[38;5;208m` (256-color orange) |
| 76–100% | Red | `\x1b[31m` (standard red) |

### Temperature Colors (per-character)

Temperature sparklines use the same color gradient principle, with degree-based thresholds:

| Temperature | Color | ANSI Code |
|-------------|-------|-----------|
| 0–49°C | Green | `\x1b[32m` (standard green) |
| 50–69°C | Yellow | `\x1b[33m` (standard yellow) |
| 70–84°C | Orange | `\x1b[38;5;208m` (256-color orange) |
| 85°C+ | Red | `\x1b[31m` (standard red) |

### Temperature Display Format

Temperature values are shown in both Celsius and Fahrenheit:

```
46°C (115°F)      — normal reading, colored by temperature
N/A°C (N/A°F)    — sensor unavailable, dim gray
```

Temperature sparklines map the range **30–100°C** to the 8 block characters (instead of 0–100% for utilization). Values below 30°C clamp to `▁`, values above 100°C clamp to `█`.

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
