# Implementation Roadmap

## Phase 1: CPU Utilization (current)

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

### What's NOT in Phase 1

- CPU temperatures
- Memory (RAM / swap)
- GPU utilization, memory, or temperature

These are displayed only as placeholder sections or omitted entirely until their respective phases.

## Phase 2: CPU Temperature

- Read thermal data from `/sys/class/hwmon/` (detect `coretemp` or `k10temp` driver)
- Add a temperature sparkline per physical core (or alongside the utilization chart)
- Define color thresholds for temperature (e.g., green < 50°C, yellow < 70°C, orange < 85°C, red >= 85°C)

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
