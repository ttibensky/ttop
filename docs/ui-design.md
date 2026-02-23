# UI Design

## Overview

ttop renders a full-screen interface using the terminal's alternate screen buffer. The display is divided into **boxed sections** stacked vertically, each representing a hardware subsystem. All metrics are displayed as **sparkline charts** with a 1-minute rolling history.

## Screen Layout

The CPU section uses a **three-column layout**: the first two thirds display utilization sparklines (CPU threads split into two side-by-side columns), and the last third displays temperature sparklines.

```
╭─ CPU ───────────────────────────────────────────────────────────────────────────────────╮
│              Utilization                  │          Temperature                         │
│ #0  ▁▂▃▄▅▆▅▄▃▂▂▃▄  52% │ #8  ▃▄▅▅▅▄▃▃  48% │ Tctl ▁▁▂▂▂▃▃▃  46°C (115°F)           │
│ #1  ▁▁▁▁▁▂▂▂▃▂▂▁▁  20% │ #9  ▁▁▁▂▂▁▁▁  15% │                                         │
│ #2  ▆▇▇█▇▇▆▇████▇  93% │ #10 ▃▄▅▅▆▅▅▄  57% │                                         │
│ #3  ▃▄▅▅▆▅▅▄▃▃▄▅▆  61% │ #11 ▁▁▂▂▃▃▂▁  30% │                                         │
│ ...                      │ ...                │                                         │
│ #7  ▃▄▅▅▅▅▄▃▃▄▅▅▅  65% │ #15 ▂▃▃▄▃▃▂▂  38% │                                         │
│                          │                    │                                         │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
╭─ Memory ────────────────────────────────────────────────────────────────────────────────╮
│      RAM Utilization       │      Swap Utilization       │        Temperature           │
│ RAM ▃▃▃▃▃▃▃▃▃  5.1GB/16.0GB  35% │ SWP ▁▁▁▁▁▁▁▁▁  0.5GB/8.0GB   6% │ N/A ▁▁▁▁  N/A°C (N/A°F)  │
│                            │                             │                              │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
╭─ GPU: NVIDIA GeForce RTX 4090 ──────────────────────────────────────────────────────────╮
│      GPU Utilization       │      VRAM Utilization       │         Temperature          │
│ USE ▅▅▅▆▆▇▆▆▅▅▅▆▆▇  72%  │ MEM ▃▃▃▃▃▃  4.2GB/24.0GB  40% │ TMP ▃▃▃▃▃▃  52°C (126°F)│
│                            │                             │                              │
╰─────────────────────────────────────────────────────────────────────────────────────────╯
                                                                        q: quit  ttop v0.1
```

**When no temperature sensors are found** (VMs, containers, etc.), the temperature column shows:

```
│  N/A ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  N/A°C (N/A°F)  │
```

## Sections

The screen is divided into independent boxed sections, stacked vertically:

1. **CPU** — split into three columns:
   - **First two thirds (Utilization):** subtitle "Utilization" (bold cyan, centered across both columns), then CPU threads split in half and displayed side by side — first half in column 1, second half in column 2, each with sparkline chart and current percentage
   - **Last third (Temperature):** subtitle "Temperature" (bold cyan, centered), then one temperature sparkline row per sensor (e.g., `Tctl` for AMD, `Core 0`–`Core N` for Intel), with dual °C/°F display
   - Vertical `│` separators divide the three columns
   - The row count is `max(ceil(cores / 2), temp_sensors)` — roughly half the height of the previous layout
   - Temperature rows are **top-aligned**: if there are fewer sensors than utilization rows, remaining temperature column rows are blank
   - If no sensors found: a single `N/A°C (N/A°F)` row with dim styling
2. **Memory** — three-column layout (equal thirds), same structure as the CPU section:
   - **First third (RAM Utilization):** subtitle "RAM Utilization" (bold cyan, centered), one sparkline row with absolute values (`usedU/totalU`) and percentage
   - **Second third (Swap Utilization):** subtitle "Swap Utilization" (bold cyan, centered), one sparkline row with absolute values and percentage
   - **Third third (Temperature):** subtitle "Temperature" (bold cyan, centered), one sparkline row per DIMM sensor (discovered via `jc42` hwmon driver) with dual °C/°F display
   - Vertical `│` separators divide the three columns
   - Row count is `max(1, dimm_sensor_count)` — RAM and SWAP always occupy row 0; extra rows appear only if multiple DIMM sensors exist
   - Temperature rows are **top-aligned**: if there are fewer sensors than rows, remaining positions are blank
   - If no DIMM sensors found: a single `N/A°C (N/A°F)` row with dim styling
   - Uses `utilization_color()` for RAM/SWAP sparklines, `temperature_color()` and `sparkline_char_temp()` for DIMM temperatures
   - When swap is disabled (`SwapTotal == 0`): SWP column renders entirely in dim gray with `0.0GB/0.0GB   0%`
3. **GPU** — three-column layout (equal thirds), same structure as the Memory section (only rendered when a GPU is detected):
   - **First third (GPU Utilization):** subtitle "GPU Utilization" (bold cyan, centered), one sparkline row with `USE` label and percentage
   - **Second third (VRAM Utilization):** subtitle "VRAM Utilization" (bold cyan, centered), one sparkline row with `MEM` label, absolute values (`usedU/totalU`), and percentage
   - **Third third (Temperature):** subtitle "Temperature" (bold cyan, centered), one sparkline row with `TMP` label and dual °C/°F display
   - Vertical `│` separators divide the three columns
   - GPU product name displayed in section title: `╭─ GPU: <name> ─╮`
   - When no GPU is detected, the entire section is omitted
   - When temperature is unavailable: TMP column shows `N/A°C (N/A°F)` with dim styling

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

### Utilization Row (first two columns)

```
│ {label} {sparkline_chart} {NNN}% │ {label} {sparkline_chart} {NNN}% │
```

- **Label:** left-aligned, fixed width — `#0`–`#N` for CPU cores (with trailing space padding), `RAM`/`SWP` for memory, `USE`/`MEM` for GPU
- **Sparkline:** variable width, fills available column space
- **Current value:** right-aligned 3-character percentage
- CPU threads are split in half: first half in column 1, second half in column 2

### Temperature Row (third column)

```
 {label} {sparkline_chart} {NNN}°C ({NNN}°F) │
```

- **Label:** right-aligned, fixed width — sensor label (e.g., `Tctl`, `Core 0`)
- **Sparkline:** variable width, fills available column space
- **Current value:** dual Celsius/Fahrenheit display, or `N/A°C (N/A°F)` if unavailable

### Memory Row (column)

```
│ {label} {sparkline_chart} {used/totalU}  {NNN}% │ {label} {sparkline_chart} {used/totalU}  {NNN}% │ {sensor} {sparkline} {NNN}°C ({NNN}°F) │
```

- **RAM/SWAP columns:** label left-aligned (3 characters — `RAM` or `SWP`), sparkline chart, absolute values (`usedU/totalU`), percentage — each in a one-third column
- **Temperature column:** sensor label right-aligned (e.g., `DIMM0`), temperature sparkline, dual °C/°F display — same layout as CPU temperature rows
- **Absolute values:** `usedU/totalU` where each value carries its own adaptive unit (KB/MB/GB/TB), allowing unambiguous display when scales differ (e.g., `8.0GB/1.0TB`)
- **Current value:** right-aligned 3-character percentage, colored by `utilization_color()`

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
