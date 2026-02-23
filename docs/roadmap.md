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

Three-column layout: utilization sparklines split across two columns (first 2/3), temperature sparklines in the third column (last 1/3).

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

## Phase 3: Memory (RAM + Swap) ✓

Full-width "Memory" section box below the CPU section, with sparkline rows for RAM and swap usage.

### Deliverables

1. **`/proc/meminfo` parser** — extract `MemTotal`, `MemAvailable`, `SwapTotal`, `SwapFree` each tick
2. **New `src/memory/` module** — `MemInfo` (raw kB values), `MemState` (rolling `VecDeque<f64>` history per metric)
3. **RAM usage** — `(MemTotal - MemAvailable) / MemTotal * 100`
4. **Swap usage** — `(SwapTotal - SwapFree) / SwapTotal * 100`
5. **Full-width Memory section** — boxed section with two sparkline rows (`RAM`, `SWP`)
6. **Absolute values** — show `usedU/totalU` alongside percentage (e.g., `5.6GB/16.0GB  35%`); each value carries its own unit so mixed scales like `8.0GB/1.0TB` are unambiguous
7. **Human-readable formatting** — `format_human_bytes()` and `format_mem_pair()` with per-value adaptive units (KB/MB/GB/TB)
8. **Color reuse** — memory sparklines use existing `utilization_color()` and `sparkline_char()`
9. **Swap-disabled handling** — when `SwapTotal == 0`, SWP row renders in dim gray with `0.0GB/0.0GB   0%`

## Phase 3.5: Memory Temperature + Three-Column Layout ✓

Redesigned Memory section from full-width to three-column layout (equal thirds): RAM Utilization, Swap Utilization, and DIMM Temperature.

### Deliverables

1. **New `src/memory/temperature.rs`** — `MemTempState` with jc42 hwmon discovery, per-DIMM temperature reading, rolling history
2. **jc42 hwmon discovery** — scan `/sys/class/hwmon/` for `name == "jc42"`, enumerate one sensor per DIMM
3. **DIMM labels** — use `temp1_label` if present, otherwise `DIMM0`, `DIMM1`, etc.
4. **Three-column layout** — equal thirds with `│` separators: RAM Utilization | Swap Utilization | Temperature
5. **Subtitle line** — three centered subtitles: "RAM Utilization", "Swap Utilization", "Temperature" (bold cyan)
6. **Temperature sparklines** — same rendering as CPU temperature (30–100°C range, `temperature_color()`, dual °C/°F display)
7. **Row count** — `max(1, dimm_sensor_count)`; RAM and SWAP always in row 0, extra rows for additional DIMMs
8. **Graceful degradation** — when no jc42 sensors found, temperature column shows `N/A°C (N/A°F)` with dim styling
9. **Color reuse** — RAM/SWAP use `utilization_color()`, temperature uses `temperature_color()` and `sparkline_char_temp()`

## Phase 4: GPU ✓

Three-column "GPU" section box below Memory, with GPU Utilization, VRAM Utilization, and Temperature each in one-third columns on a single line. GPU name displayed in the section title. Section hidden entirely when no GPU is detected.

### Deliverables

1. **Vendor detection** — try NVIDIA first (via `nvidia-smi`), then AMD (via `/sys/class/drm/` vendor ID `0x1002`)
2. **New `src/gpu/` module** — `GpuState` (rolling `VecDeque<f64>` history for util/mem/temp), `GpuBackend` enum dispatching to vendor-specific readers
3. **NVIDIA backend (`src/gpu/nvidia.rs`)** — `detect()` queries `nvidia-smi --query-gpu=name`; `read_snapshot()` queries utilization, memory used/total, and temperature in a single call
4. **AMD backend (`src/gpu/amd.rs`)** — `detect()` scans `/sys/class/drm/card*/device/vendor` for `0x1002`; reads `gpu_busy_percent`, `mem_info_vram_used/total` from sysfs; temperature via hwmon `amdgpu` driver
5. **GPU name in title** — section header renders as `╭─ GPU: <name> ─╮`
6. **Three-column layout** — `USE` (utilization %) in first third, `MEM` (memory % + absolute values) in second third, `TMP` (temperature °C/°F) in last third, separated by `│` dividers with centered bold cyan subtitles
7. **Per-column chart widths** — each column computes its own chart width using existing `util_chart_width`, `mem_col_chart_width`, and `temp_chart_width` functions; histories trimmed to the maximum of all three
8. **Color reuse** — USE and MEM columns use `utilization_color()`; TMP column uses `temperature_color()` and `sparkline_char_temp()`
9. **Graceful degradation** — section not rendered when no GPU detected; temperature column shows `N/A°C (N/A°F)` if no hwmon found

## Phase 5: Disk (Space + I/O) ✓

- New `src/disk/` module with `DiskSpaceState` and `DiskIoState` structs
- **Space usage** (left half):
  - Parse `/proc/mounts` to discover real filesystems (filter out `tmpfs`, `sysfs`, `proc`, etc.)
  - Call `libc::statvfs` on each mount point for used/total bytes
  - One sparkline row per filesystem, labeled by mount point (e.g., `/`, `/home`)
  - Display: `used/totalG  NN%`
- **I/O throughput** (right half):
  - Parse `/proc/diskstats` for cumulative sector read/write counters
  - Delta computation between ticks to derive bytes/sec per device (sector = 512 bytes)
  - Filter to whole-disk devices (exclude partitions like `sda1`)
  - Separate R (read) and W (write) sparkline rows per device
  - Auto-scaling sparklines (0 to max observed value) via new `sparkline_char_scaled()` helper
  - Display: human-readable rate with adaptive unit (`KB/s`, `MB/s`, `GB/s`)
- Side-by-side "Disk" section box (same layout pattern as CPU)
- Add `libc` as direct dependency (already a transitive dep of `crossterm`, zero cost)

## Phase 6: Polish

- Graceful handling of missing hardware (no GPU, no temp sensors)
- Error resilience (permission denied, file not found, malformed data)
- Command-line arguments (refresh rate override, disable sections, etc.)
- Man page or `--help` documentation

## Phase 7: PPA Distribution

Publish ttop as a PPA so users can install it with `sudo apt install ttop`.

### Deliverables

1. **Debian packaging (`debian/` directory)**
   - `debian/control` — package name (`ttop`), description, architecture (`amd64`), build-deps (`cargo`, `rustc`, `debhelper-compat`)
   - `debian/rules` — `dh`-based build using `cargo build --release`, install binary to `/usr/bin/ttop`
   - `debian/changelog` — version tracking in Debian format
   - `debian/copyright` — machine-readable copyright file (AGPL-3.0)
   - `debian/install` — maps `target/release/ttop` to `/usr/bin/`
   - `debian/source/format` — `3.0 (quilt)` or `3.0 (native)`
2. **Launchpad PPA setup**
   - Create PPA on Launchpad (e.g., `ppa:ttibensky/ttop`)
   - GPG key generation and upload to Launchpad / Ubuntu keyserver
   - Target releases: Ubuntu 24.04 LTS (Noble) and latest non-LTS (currently 25.10 Oracular)
3. **CI automation (GitHub Actions)**
   - Workflow triggered on version tag push (e.g., `v*`)
   - Build source package (`debuild -S`)
   - Upload to PPA via `dput`
   - GPG signing key stored as GitHub Actions secret
4. **README update** — add `apt install` instructions to the Installation section

## Phase 8: Performance Benchmarks

Run a resource-usage comparison of `ttop` vs `top` vs `htop` and publish the results in the README.

### Deliverables

1. **Benchmark harness** — script that launches each tool under identical conditions and collects CPU usage, RSS memory, and startup time
2. **Sampling methodology** — run each tool for a fixed duration (e.g., 60 seconds), sample `/proc/<pid>/stat` and `/proc/<pid>/status` at regular intervals
3. **Metrics collected** — average CPU %, peak RSS (kB), startup-to-first-render latency
4. **Environment normalization** — document system specs (CPU, RAM, kernel version), ensure idle baseline, pin refresh rates to 1 second across all three tools
5. **Results table in README** — add a "Performance" section with a comparison table (tool, avg CPU %, peak RSS, startup time) and a brief interpretation
6. **Reproducibility instructions** — include the benchmark script and usage instructions so others can replicate the results on their own hardware

## Design Decisions Log

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Language | Rust | Performance, safety, single binary distribution |
| Terminal library | `crossterm` | Minimal, cross-platform terminal control without a full TUI framework |
| Disk space query | `libc::statvfs` via `libc` crate | Direct syscall, already a transitive dep of `crossterm` (zero cost); no need for a higher-level crate |
| Data source | `/proc/stat`, `/sys/class/hwmon/` (coretemp/k10temp, jc42, amdgpu), `/proc/meminfo`, `nvidia-smi`, `/sys/class/drm/`, `/proc/mounts`, `statvfs`, `/proc/diskstats` | Kernel interfaces + vendor CLI where sysfs is unavailable |
| Chart type | Single-row sparklines (`▁▂▃▄▅▆▇█`) | Compact enough to show all cores on one screen, 8 levels of vertical resolution per row |
| Chart width | Dynamic (fills terminal width) | Wider terminals show more history; adapts on resize |
| Color scheme | Green → Yellow → Orange → Red | Intuitive severity gradient, readable on dark backgrounds |
| Color per character | Each sparkline character independently colored | Creates a natural heatmap showing load transitions over time |
| Refresh rate | 1 second | Matches `/proc/stat` granularity, consistent with `top` defaults |
| History depth | Equal to chart width (1 char = 1 second) | 1:1 mapping keeps the mental model simple |
| Layout | Vertically stacked boxed sections | Clean visual separation, easy to extend with new widgets |
| Screen mode | Full-screen (alternate buffer) | Clean experience, restores terminal on exit |
| Package distribution | Launchpad PPA | Native `apt install` experience on Ubuntu; automated builds for multiple releases; no hosting infrastructure needed |
