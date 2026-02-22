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

### CPU Temperature — `/sys/class/hwmon/` (future)

Thermal data is exposed via the hardware monitoring subsystem:

```
/sys/class/hwmon/hwmonN/temp1_input   # temperature in millidegrees Celsius
/sys/class/hwmon/hwmonN/temp1_label   # sensor label (e.g., "Core 0")
/sys/class/hwmon/hwmonN/name          # driver name (e.g., "coretemp")
```

The correct hwmon device is identified by finding the one with `name == "coretemp"` (Intel) or `name == "k10temp"` (AMD). Individual core temperatures are then read from `tempN_input` files.

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

### Main Loop

```
initialize terminal (alternate screen, raw mode, hide cursor)
take initial /proc/stat snapshot

loop every 1 second:
    poll for key events (non-blocking)
        if 'q' or Ctrl+C → break

    read /proc/stat
    compute per-core deltas from previous snapshot
    push new values into history buffers
    store current snapshot for next iteration

    calculate layout dimensions from terminal size
    render all sections to a string buffer
    flush buffer to terminal

restore terminal (show cursor, disable raw mode, leave alternate screen)
```

### Data Model

```
struct CoreHistory {
    samples: VecDeque<f64>,   // rolling buffer, max size = chart width
}

struct CpuState {
    prev_snapshot: Vec<CpuTimes>,    // previous /proc/stat reading
    histories: Vec<CoreHistory>,      // one per logical processor
}

struct CpuTimes {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    iowait: u64,
    irq: u64,
    softirq: u64,
    steal: u64,
}
```

The `VecDeque` acts as a ring buffer. When a new sample is pushed and the buffer exceeds the current chart width, the oldest sample is popped from the front.

### Rendering Pipeline

1. **Query terminal size** — get current column and row count
2. **Calculate layout** — determine chart width by subtracting fixed elements (labels, padding, borders) from terminal width
3. **Resize buffers** — if terminal width changed, grow or shrink history buffers
4. **Build frame** — iterate over each section and row, converting data points to colored sparkline characters
5. **Output frame** — move cursor to top-left, write the complete frame buffer to stdout in a single flush

The frame is composed in a `String` buffer before writing to minimize flickering. Cursor positioning uses ANSI escape codes (`\x1b[H` to home, `\x1b[K` to clear line remainders).

### Color Selection

A simple function maps a percentage value to an ANSI color code:

```
fn utilization_color(pct: f64) -> &str {
    match pct {
        0.0..=25.0   => "\x1b[32m",          // green
        25.0..=50.0  => "\x1b[33m",          // yellow
        50.0..=75.0  => "\x1b[38;5;208m",    // orange (256-color)
        _            => "\x1b[31m",          // red
    }
}
```

### Sparkline Character Selection

A percentage is mapped to one of 8 Unicode block elements:

```
fn sparkline_char(pct: f64) -> char {
    const CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
    let index = ((pct / 100.0) * 7.0).round() as usize;
    CHARS[index.min(7)]
}
```

Each character is wrapped with its own color code, so adjacent characters in the same sparkline can have different colors.
