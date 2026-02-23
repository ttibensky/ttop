# Technical Architecture

## Overview

ttop is a single-binary Rust application that reads system metrics from Linux kernel interfaces, maintains a rolling history buffer, and renders a full-screen TUI using ANSI escape codes via the `crossterm` crate.

## Dependencies

| Crate | Purpose |
|-------|---------|
| `crossterm` | Terminal control: alternate screen buffer, raw mode, cursor hiding, key event polling, 256-color output |

Everything else uses the Rust standard library. No system monitoring crates, no TUI frameworks.

## Data Sources

All data is read from virtual filesystems provided by the Linux kernel. No external commands are spawned.

### CPU Utilization — `/proc/stat`

The kernel exposes cumulative CPU time counters in `/proc/stat`:

```
cpu  10132153 290696 3084719 46828483 16683 0 25195 0 0 0
cpu0  1393280  32966  572056  13343292  6130  0  17875  0  0  0
cpu1  1335232  28612  521789  13287858  4720  0   3556  0  0  0
...
```

Each `cpuN` line contains these fields (in "jiffies" / clock ticks):

| Field | Index | Description |
|-------|-------|-------------|
| user | 0 | Time in user mode |
| nice | 1 | Time in user mode (low priority) |
| system | 2 | Time in kernel mode |
| idle | 3 | Time idle |
| iowait | 4 | Time waiting for I/O |
| irq | 5 | Time servicing hardware interrupts |
| softirq | 6 | Time servicing software interrupts |
| steal | 7 | Time stolen by hypervisor |

**Algorithm to compute utilization:**

1. Read `/proc/stat` at time T1, parse all `cpuN` lines
2. Wait 1 second
3. Read `/proc/stat` at time T2, parse all `cpuN` lines
4. For each core, compute deltas:
   - `idle_delta = (idle_T2 - idle_T1) + (iowait_T2 - iowait_T1)`
   - `total_delta = sum(all_fields_T2) - sum(all_fields_T1)`
   - `usage = (total_delta - idle_delta) / total_delta * 100`
5. Store the usage percentage in the rolling history buffer

The number of `cpuN` lines is detected dynamically — works for any core count.

### CPU Temperature — `/sys/class/hwmon/`

Thermal data is exposed via the hardware monitoring subsystem:

```
/sys/class/hwmon/hwmonN/temp1_input   # temperature in millidegrees Celsius
/sys/class/hwmon/hwmonN/temp1_label   # sensor label (e.g., "Core 0", "Tctl")
/sys/class/hwmon/hwmonN/name          # driver name (e.g., "coretemp", "k10temp")
```

**Discovery algorithm:**

1. Iterate over `/sys/class/hwmon/hwmon*/`
2. Read the `name` file in each directory
3. Match on `k10temp` (AMD) or `coretemp` (Intel)
4. For the matched device, enumerate all `tempN_input` / `tempN_label` file pairs
5. Store the paths for repeated reading each tick

**AMD vs Intel differences:**

- **AMD (`k10temp`):** exposes a single package temperature sensor labeled `Tctl`. Per-core temperatures are not available. The right half of the CPU section shows one temperature row.
- **Intel (`coretemp`):** exposes per-core temperature sensors labeled `Core 0`, `Core 1`, etc. The right half shows one row per physical core.

Temperature values in sysfs are in millidegrees Celsius (e.g., `46375` = 46.375°C). Displayed in both Celsius and Fahrenheit: `46°C (115°F)`.

**Graceful degradation:** when no matching hwmon device is found (VMs, containers, unsupported hardware), the temperature column displays `N/A°C (N/A°F)` with dim styling.

**Integrated GPUs:** even on AMD APUs where the GPU is on the same die, Linux exposes CPU and GPU temperatures as separate hwmon devices (`k10temp` for CPU, `amdgpu` for GPU). They are handled by different modules: `src/cpu/temperature.rs` and future `src/gpu/temperature.rs`.

### Memory — `/proc/meminfo` (future)

```
MemTotal:       16384000 kB
MemAvailable:    8192000 kB
SwapTotal:       8192000 kB
SwapFree:        7700000 kB
```

- RAM usage: `(MemTotal - MemAvailable) / MemTotal * 100`
- Swap usage: `(SwapTotal - SwapFree) / SwapTotal * 100`

### GPU — vendor-specific (future)

- **NVIDIA:** read from `nvidia-smi --query-gpu=utilization.gpu,utilization.memory,temperature.gpu --format=csv,noheader,nounits` or the NVML sysfs interface
- **AMD:** read from `/sys/class/drm/card0/device/gpu_busy_percent` and related sysfs files

## Application Structure

### Module Layout

```
src/
├── main.rs               # entry point, terminal init, event loop
├── cpu/
│   ├── mod.rs            # re-exports CpuState, TempState
│   ├── utilization.rs    # /proc/stat parsing, per-core usage history
│   └── temperature.rs    # hwmon discovery, sysfs temp reading, history
└── ui.rs                 # rendering: layout, sparklines, colors, frame composition
```

### Main Loop

```
initialize terminal (alternate screen, raw mode, hide cursor)
take initial /proc/stat snapshot
discover temperature sensors via hwmon

loop every 1 second:
    poll for key events (non-blocking)
        if 'q' or Ctrl+C → break

    read /proc/stat → compute per-core utilization deltas
    read hwmon tempN_input → current temperatures
    push new values into history buffers

    calculate layout dimensions (left/right halves from terminal size)
    render all sections to a string buffer
    flush buffer to terminal

restore terminal (show cursor, disable raw mode, leave alternate screen)
```

### Data Model

```
struct CpuTimes {
    user, nice, system, idle, iowait, irq, softirq, steal: u64,
}

struct CpuState {
    prev_snapshot: Vec<CpuTimes>,
    histories: Vec<VecDeque<f64>>,     // one per logical processor
}

struct TempSensor {
    input_path: PathBuf,               // e.g. /sys/class/hwmon/hwmon3/temp1_input
    label: String,                     // e.g. "Tctl", "Core 0"
}

struct TempState {
    sensors: Vec<TempSensor>,
    histories: Vec<VecDeque<f64>>,     // one per sensor, values in °C
}
```

The `VecDeque` acts as a ring buffer. When a new sample is pushed and the buffer exceeds the current chart width, the oldest sample is popped from the front.

### Rendering Pipeline

1. **Query terminal size** — get current column and row count
2. **Calculate layout** — split terminal into left half (utilization) and right half (temperature), determine chart widths for each by subtracting fixed elements (labels, padding, borders, temp display) from half-widths
3. **Resize buffers** — if terminal width changed, grow or shrink history buffers
4. **Build frame** — iterate over each row; render utilization sparkline on the left, vertical separator, and temperature sparkline on the right; top-align temperature rows (fill remaining right-half rows with empty space)
5. **Output frame** — move cursor to top-left, write the complete frame buffer to stdout in a single flush

The frame is composed in a `String` buffer before writing to minimize flickering. Cursor positioning uses ANSI escape codes (`\x1b[H` to home, `\x1b[K` to clear line remainders).

### Side-by-Side Layout

```
╭─ CPU ───────────────────────────────────────────────────────────────────────╮
│                                    │                                       │
│  0 ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▂  3% │ Tctl ▁▁▁▁▁▁▁▁▁▁▁▁▁▃  46°C (115°F)  │
│  1 ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  0% │                                       │
│  2 ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁  1% │                                       │
│  ...                              │                                       │
│                                    │                                       │
╰─────────────────────────────────────────────────────────────────────────────╯
```

The left and right halves are separated by a thin `│` vertical separator. Temperature rows are top-aligned: if there are fewer sensors than CPU cores, the remaining right-half rows are blank. If no sensors are found, a single `N/A°C (N/A°F)` row appears.

### Color Selection

**Utilization colors** (by percentage):

| Range | Color |
|-------|-------|
| 0–25% | Green |
| 26–50% | Yellow |
| 51–75% | Orange (256-color) |
| 76–100% | Red |

**Temperature colors** (by °C):

| Range | Color |
|-------|-------|
| 0–49°C | Green |
| 50–69°C | Yellow |
| 70–84°C | Orange (256-color) |
| 85°C+ | Red |

### Sparkline Character Selection

Utilization maps 0–100% to one of 8 Unicode block elements. Temperature maps 30–100°C to the same 8 characters:

```
const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
```

Each character is wrapped with its own color code, so adjacent characters in the same sparkline can have different colors.

### Temperature Display Format

Each temperature sensor shows the current value in both Celsius and Fahrenheit:

```
46°C (115°F)     — normal sensor reading
N/A°C (N/A°F)   — sensor unavailable (dim styling)
```
