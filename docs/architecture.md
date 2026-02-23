# Technical Architecture

## Overview

ttop is a Rust application structured as a **library + binary** crate. The library (`src/lib.rs`) exposes all core logic — data collection, history management, and rendering — while the binary (`src/main.rs`) handles terminal setup and the event loop. This split allows all tests to live as external integration tests in `tests/`, importing the public API via `use ttop::...`.

The application reads system metrics from Linux kernel interfaces, maintains a rolling history buffer, and renders a full-screen TUI using ANSI escape codes via the `crossterm` crate.

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

- **AMD (`k10temp`):** exposes a single package temperature sensor labeled `Tctl`. Per-core temperatures are not available. The temperature column shows one temperature row.
- **Intel (`coretemp`):** exposes per-core temperature sensors labeled `Core 0`, `Core 1`, etc. The temperature column shows one row per physical core.

Temperature values in sysfs are in millidegrees Celsius (e.g., `46375` = 46.375°C). Displayed in both Celsius and Fahrenheit: `46°C (115°F)`.

**Graceful degradation:** when no matching hwmon device is found (VMs, containers, unsupported hardware), the temperature column displays `N/A°C (N/A°F)` with dim styling.

**Integrated GPUs:** even on AMD APUs where the GPU is on the same die, Linux exposes CPU and GPU temperatures as separate hwmon devices (`k10temp` for CPU, `amdgpu` for GPU). They are handled by different modules: `src/cpu/temperature.rs` and future `src/gpu/temperature.rs`.

### Memory — `/proc/meminfo`

The kernel exposes memory statistics in `/proc/meminfo`:

```
MemTotal:       16384000 kB
MemAvailable:    8192000 kB
SwapTotal:       8192000 kB
SwapFree:        7700000 kB
```

The parser reads the file each tick and extracts four fields by key name. Values are in kilobytes.

- RAM usage: `(MemTotal - MemAvailable) / MemTotal * 100`
- Swap usage: `(SwapTotal - SwapFree) / SwapTotal * 100`

**Swap-disabled handling:** when `SwapTotal == 0`, the swap percentage is 0 and the SWP row renders in dim gray with `0.0GB/0.0GB   0%`.

### GPU — vendor-specific

GPU data comes from vendor-specific interfaces:

**NVIDIA** — reads from `nvidia-smi` (the standard NVIDIA management CLI):

- Detection: `nvidia-smi --query-gpu=name --format=csv,noheader,nounits`
- Per-tick read: `nvidia-smi --query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu --format=csv,noheader,nounits`
- Returns: utilization %, memory used/total in MiB, temperature in °C

**AMD** — reads from kernel sysfs interfaces:

- Detection: scan `/sys/class/drm/card*/device/vendor` for AMD vendor ID `0x1002`
- Utilization: `/sys/class/drm/cardN/device/gpu_busy_percent` (0–100)
- Memory: `/sys/class/drm/cardN/device/mem_info_vram_used` and `mem_info_vram_total` (bytes)
- Temperature: hwmon device with `name == "amdgpu"`, reading `temp1_input` (millidegrees)
- GPU name: `/sys/class/drm/cardN/device/product_name`

**Graceful degradation:** when no GPU is detected (no NVIDIA driver, no AMD card), the GPU section is not rendered at all.

## Application Structure

### Module Layout

```
src/
├── lib.rs                # library crate root, re-exports cpu, gpu, memory, and ui modules
├── main.rs               # binary entry point, terminal init, event loop
├── cpu/
│   ├── mod.rs            # re-exports CpuState, CpuTimes, TempState
│   ├── utilization.rs    # /proc/stat parsing, per-core usage history
│   └── temperature.rs    # hwmon discovery, sysfs temp reading, history
├── gpu/
│   ├── mod.rs            # GpuState, GpuBackend enum, vendor detection dispatch
│   ├── nvidia.rs         # NVIDIA detection and reading via nvidia-smi
│   └── amd.rs            # AMD detection and reading via DRM/hwmon sysfs
├── memory/
│   ├── mod.rs            # re-exports MemState, MemInfo
│   └── usage.rs          # /proc/meminfo parsing, RAM+swap usage history
└── ui.rs                 # rendering: layout, sparklines, colors, frame composition

tests/
├── cpu_temperature.rs    # temperature module tests
├── cpu_utilization.rs    # CPU utilization module tests
├── gpu.rs                # GPU module tests
├── memory_usage.rs       # memory usage module tests
└── ui.rs                 # UI rendering tests
```

All tests are external integration tests that import from the `ttop` library crate. Test files mirror the source module they cover.

### Main Loop

```
initialize terminal (alternate screen, raw mode, hide cursor)
take initial /proc/stat snapshot
discover temperature sensors via hwmon
take initial /proc/meminfo snapshot
detect GPU vendor (NVIDIA via nvidia-smi, AMD via /sys/class/drm/)

loop every 1 second:
    poll for key events (non-blocking)
        if 'q' or Ctrl+C → break

    read /proc/stat → compute per-core utilization deltas
    read hwmon tempN_input → current temperatures
    read /proc/meminfo → compute RAM and swap usage percentages
    read GPU metrics (nvidia-smi or sysfs) → utilization, memory, temperature
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

struct MemInfo {
    mem_total_kb: u64,
    mem_available_kb: u64,
    swap_total_kb: u64,
    swap_free_kb: u64,
}

struct MemState {
    ram_history: VecDeque<f64>,        // RAM usage percentage
    swap_history: VecDeque<f64>,       // swap usage percentage
    current: MemInfo,                  // latest raw values for absolute display
}

enum GpuBackend {
    Nvidia,
    Amd { card_path: PathBuf, hwmon_path: Option<PathBuf> },
    None,
}

struct GpuState {
    backend: GpuBackend,               // vendor-specific reader
    name: String,                      // GPU product name (shown in section title)
    util_history: VecDeque<f64>,       // GPU utilization percentage
    mem_history: VecDeque<f64>,        // GPU memory usage percentage
    temp_history: VecDeque<f64>,       // GPU temperature in °C
    current_mem_used_kb: u64,          // for absolute memory display
    current_mem_total_kb: u64,
}
```

The `VecDeque` acts as a ring buffer. When a new sample is pushed and the buffer exceeds the current chart width, the oldest sample is popped from the front.

### Rendering Pipeline

1. **Query terminal size** — get current column and row count
2. **Calculate layout** — split terminal into three columns (two utilization columns at 2/3 width, one temperature column at 1/3 width), determine chart widths by subtracting fixed elements (labels, padding, borders, temp display) from column widths
3. **Resize buffers** — if terminal width changed, grow or shrink history buffers
4. **Build frame** — split CPU threads in half; iterate over rows, rendering first-half utilization in column 1, second-half in column 2, and temperature in column 3; top-align all three columns
5. **Output frame** — move cursor to top-left, write the complete frame buffer to stdout in a single flush

The frame is composed in a `String` buffer before writing to minimize flickering. Cursor positioning uses ANSI escape codes (`\x1b[H` to home, `\x1b[K` to clear line remainders).

For the visual layout, color scheme, sparkline character set, and row formatting details, see [docs/ui-design.md](ui-design.md).
