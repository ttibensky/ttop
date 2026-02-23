# Performance Comparison: ttop vs top vs htop

## Test Environment

| Property | Value |
|----------|-------|
| Laptop | Lenovo IdeaPad Pro 5 14APH8 |
| CPU | AMD Ryzen 7 7840HS with Radeon 780M Graphics (16 logical cores) |
| RAM | 32.0 GiB |
| OS | Ubuntu 25.10 |
| Kernel | 6.17.0-14-generic |
| ttop | built from source (release mode, `cargo build --release`) |
| top | procps-ng (system default) |
| htop | system package (`apt install htop`) |

## Methodology

Each tool was benchmarked independently under identical conditions. The three tools were run **sequentially** (never concurrently) to avoid interference. The system was otherwise idle during testing.

### Procedure

1. **Build ttop** in release mode (`cargo build --release`) to ensure optimized performance.
2. **Launch one tool at a time** with a 1-second refresh rate:
   - `ttop` — default 1-second refresh, launched via `script -qfc` for pseudo-terminal allocation.
   - `top -d 1` — 1-second refresh, launched via `script -qfc` for consistent PTY environment.
   - `htop -d 10` — 1-second refresh (`-d` is in tenths of a second), launched via `script -qfc`.
3. **Warm-up period**: after launching, wait 2 seconds before sampling to let the tool stabilize (initial rendering, process enumeration, etc.).
4. **Sample for 60 seconds** at 1-second intervals, collecting:
   - **CPU usage**: read fields 14 (`utime`) and 15 (`stime`) from `/proc/<pid>/stat`. These are cumulative clock ticks spent in user and kernel mode. CPU percentage for each interval is computed as:
     ```
     cpu% = (delta_utime + delta_stime) / (delta_wall_seconds × CLK_TCK) × 100
     ```
     where `CLK_TCK` is the system clock tick rate (typically 100 on Linux).
   - **Memory usage (RSS)**: read `VmRSS` from `/proc/<pid>/status`, which reports the resident set size in kB — the actual physical memory occupied by the process.
5. **Compute averages** over all 60 samples.
6. **Kill the tool** and move on to the next one.

### Why These Metrics

- **Average CPU %** reflects the ongoing processing cost of running the monitor. Lower is better — a system monitor should consume minimal CPU to avoid perturbing the measurements it displays.
- **Average RSS (MB)** measures actual physical memory footprint. Lower is better — especially on memory-constrained systems.

### Fairness Measures

- All tools run with the same 1-second refresh interval.
- All tools are launched via `script -qfc` to provide a consistent pseudo-terminal environment.
- Tools run sequentially with no other significant workloads.
- The same 2-second warm-up is applied to each tool.
- Delta-based CPU measurement avoids artifacts from process startup.

## Results

| Tool | Avg CPU (%) | Avg Memory (MB) |
|------|-------------|-----------------|
| ttop | 0.14 | 3.1 |
| top  | 1.25 | 5.9 |
| htop | 8.68 | 7.8 |

### Comparison (ttop vs others)

| Metric | ttop vs top | ttop vs htop |
|--------|-------------|--------------|
| CPU    | -88.8% | -98.3% |
| Memory | -47.8% | -60.0% |

_Negative percentages mean ttop uses fewer resources; positive means it uses more._

### Interpretation

ttop uses significantly fewer resources than both top and htop. At 0.14% average CPU, it consumes roughly 9x less CPU than top and 62x less than htop. Memory usage follows a similar pattern — ttop's 3.1 MB RSS footprint is about half of top's and 2.5x smaller than htop's.

This is not an apples-to-apples comparison. Both top and htop enumerate, sort, and render the entire process table on every refresh cycle, which is inherently more work than what ttop does. ttop reads a small, fixed set of kernel files (`/proc/stat`, `/proc/meminfo`, `/sys/class/hwmon/`, etc.) once per second and maintains only fixed-size rolling history buffers — it has no per-process data structures at all.

However, the comparison is still practically relevant. Most of the time when you reach for a system monitor, you just want a quick overview of CPU, memory, GPU, and disk activity — not a scrollable process list. For that use case, ttop delivers the information at a fraction of the resource cost.

## How to Reproduce

### Prerequisites

```bash
sudo apt install procps htop util-linux bc
```

All of these are typically pre-installed on Ubuntu.

### Running the Benchmark

From the project root:

```bash
# Build and benchmark (takes ~3.5 minutes)
./docs/benchmark/benchmark.sh
```

The script will:
1. Build ttop in release mode.
2. Run each tool for 60 seconds with 1-second sampling.
3. Print results to stdout and save them to `docs/benchmark/results.md`.

### Customization

Edit the variables at the top of [`docs/benchmark/benchmark.sh`](benchmark/benchmark.sh) to change:
- `DURATION` — how long each tool runs (default: 60 seconds)
- `SAMPLE_INTERVAL` — sampling frequency (default: 1 second)
- `WARMUP` — stabilization delay before sampling starts (default: 2 seconds)

## Benchmark Script

The full benchmark script is available at [`docs/benchmark/benchmark.sh`](benchmark/benchmark.sh).
