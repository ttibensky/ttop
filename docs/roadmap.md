# Implementation Roadmap

## Phase 1: CPU Utilization ✓

The first milestone — a working full-screen app that displays per-core CPU usage as sparkline charts.

### Deliverables

1. **Project scaffolding** — `cargo init`, add `crossterm` dependency
2. **`/proc/stat` parser** — read and parse per-core CPU time counters
3. **Delta computation** — calculate per-core utilization percentage from two consecutive snapshots
4. **History buffer** — rolling `VecDeque` per core, sized to chart width
5. **Sparkline rendering** — map values to `▁▂▃▄▅▆▇█` characters with color coding
6. **Layout engine** — box-drawing borders, dynamic chart width based on terminal size
7. **Main loop** — 1-second tick, key event polling (`q` / Ctrl+C to exit), full-screen alternate buffer
8. **Terminal resize handling** — recalculate layout and resize buffers on `SIGWINCH`

## Phase 2: CPU Temperature ✓

Side-by-side layout: utilization sparklines on the left half, temperature sparklines on the right half.

### Deliverables

1. **Module restructure** — refactored `src/cpu.rs` into `src/cpu/mod.rs`, `src/cpu/utilization.rs`, `src/cpu/temperature.rs`
2. **hwmon discovery** — auto-detect `k10temp` (AMD) or `coretemp` (Intel) driver under `/sys/class/hwmon/`
3. **Sensor enumeration** — discover all `tempN_input` / `tempN_label` pairs for the matched device
4. **Temperature reading** — read millidegree values from sysfs each tick, convert to °C
5. **History buffers** — rolling `VecDeque<f64>` per sensor, same as utilization
6. **Side-by-side layout** — CPU widget split in half with `│` vertical separator
7. **Temperature sparklines** — range 30–100°C, same block characters as utilization
8. **Temperature colors** — green (< 50°C), yellow (50–69°C), orange (70–84°C), red (≥ 85°C)
9. **Dual display format** — `46°C (115°F)` next to each temp sparkline
10. **N/A fallback** — `N/A°C (N/A°F)` with dim styling when no sensors found
11. **Top-aligned rows** — temperature rows top-aligned on right; empty space below if fewer sensors than cores

## Phase 3: Memory

- Parse `/proc/meminfo` for RAM and swap
- Add the Memory section box with two sparkline rows (RAM, SWP)
- Show absolute values alongside percentages (e.g., `5.6/16G`)

## Phase 4: GPU

- Detect GPU vendor (NVIDIA / AMD / Intel)
- NVIDIA: query `nvidia-smi` or NVML sysfs interface
- AMD: read from `/sys/class/drm/` sysfs files
- Add the GPU section box with utilization, memory, and temperature sparklines
- GPU name displayed in the section title

## Phase 5: Polish

- Graceful handling of missing hardware (no GPU, no temp sensors)
- Error resilience (permission denied, file not found, malformed data)
- Command-line arguments (refresh rate override, disable sections, etc.)
- Man page or `--help` documentation

## Design Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance, safety, single binary distribution |
| Terminal library | `crossterm` | Minimal, cross-platform terminal control without a full TUI framework |
| Data source | `/proc/stat`, `/sys/class/hwmon/`, `/proc/meminfo` | Direct kernel interfaces, zero runtime dependencies |
| Chart type | Single-row sparklines (`▁▂▃▄▅▆▇█`) | Compact enough to show all cores on one screen, 8 levels of vertical resolution per row |
| Chart width | Dynamic (fills terminal width) | Wider terminals show more history; adapts on resize |
| Color scheme | Green → Yellow → Orange → Red | Intuitive severity gradient, readable on dark backgrounds |
| Color per character | Each sparkline character independently colored | Creates a natural heatmap showing load transitions over time |
| Refresh rate | 1 second | Matches `/proc/stat` granularity, consistent with `top` defaults |
| History depth | Equal to chart width (1 char = 1 second) | 1:1 mapping keeps the mental model simple |
| Layout | Vertically stacked boxed sections | Clean visual separation, easy to extend with new widgets |
| Screen mode | Full-screen (alternate buffer) | Clean experience, restores terminal on exit |
