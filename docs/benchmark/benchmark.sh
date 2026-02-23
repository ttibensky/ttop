#!/usr/bin/env bash
set -euo pipefail

DURATION=60
SAMPLE_INTERVAL=1
WARMUP=2
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TTOP_BIN="$PROJECT_ROOT/target/release/ttop"
RESULTS_FILE="$SCRIPT_DIR/results.md"

CLK_TCK=$(getconf CLK_TCK)

check_prerequisites() {
    local missing=()
    command -v top   &>/dev/null || missing+=("top (install: sudo apt install procps)")
    command -v htop  &>/dev/null || missing+=("htop (install: sudo apt install htop)")
    command -v script &>/dev/null || missing+=("script (install: sudo apt install util-linux)")
    command -v bc    &>/dev/null || missing+=("bc (install: sudo apt install bc)")

    if [[ ${#missing[@]} -gt 0 ]]; then
        echo "ERROR: missing required commands:" >&2
        printf "  - %s\n" "${missing[@]}" >&2
        exit 1
    fi
}

build_ttop() {
    echo "Building ttop (release)..." >&2
    (cd "$PROJECT_ROOT" && cargo build --release --quiet)
    if [[ ! -x "$TTOP_BIN" ]]; then
        echo "ERROR: ttop binary not found at $TTOP_BIN" >&2
        exit 1
    fi
}

# Read utime+stime (clock ticks) from /proc/<pid>/stat
read_cpu_ticks() {
    local pid=$1
    awk '{print $14 + $15}' "/proc/$pid/stat"
}

# Read VmRSS (kB) from /proc/<pid>/status
read_rss_kb() {
    local pid=$1
    awk '/^VmRSS:/ {print $2}' "/proc/$pid/status"
}

# Launch a tool, sample it for DURATION seconds, print "avg_cpu avg_rss_kb"
benchmark_tool() {
    local name=$1
    shift
    local cmd=("$@")

    echo "Benchmarking $name for ${DURATION}s (+ ${WARMUP}s warm-up)..." >&2

    # Launch the tool in the background with a pseudo-terminal
    script -qfc "${cmd[*]}" /dev/null &>/dev/null &
    local wrapper_pid=$!
    sleep "$WARMUP"

    # Find the actual tool PID (descendant of the script wrapper)
    local tool_pid
    tool_pid=$(pgrep -x "$(basename "${cmd[0]}")" --newest) || true

    if [[ -z "$tool_pid" ]] || ! kill -0 "$tool_pid" 2>/dev/null; then
        echo "  ERROR: could not find running $name process" >&2
        kill "$wrapper_pid" 2>/dev/null || true
        wait "$wrapper_pid" 2>/dev/null || true
        echo "0 0"
        return
    fi

    echo "  PID: $tool_pid" >&2

    local prev_ticks now_ns prev_ns
    prev_ticks=$(read_cpu_ticks "$tool_pid")
    prev_ns=$(date +%s%N)

    local total_cpu="0" total_rss=0 samples=0

    for ((i = 0; i < DURATION; i++)); do
        sleep "$SAMPLE_INTERVAL"

        if ! kill -0 "$tool_pid" 2>/dev/null; then
            echo "  WARNING: $name exited early after $i seconds" >&2
            break
        fi

        local cur_ticks cur_ns
        cur_ticks=$(read_cpu_ticks "$tool_pid")
        cur_ns=$(date +%s%N)

        local delta_ticks=$(( cur_ticks - prev_ticks ))
        local delta_ns=$(( cur_ns - prev_ns ))
        local cpu_pct
        cpu_pct=$(echo "scale=4; $delta_ticks / ($delta_ns / 1000000000) / $CLK_TCK * 100" | bc)

        local rss_kb
        rss_kb=$(read_rss_kb "$tool_pid")

        total_cpu=$(echo "$total_cpu + $cpu_pct" | bc)
        total_rss=$(( total_rss + rss_kb ))
        samples=$(( samples + 1 ))

        prev_ticks=$cur_ticks
        prev_ns=$cur_ns
    done

    # Clean up
    kill "$wrapper_pid" 2>/dev/null || true
    wait "$wrapper_pid" 2>/dev/null || true

    if [[ $samples -eq 0 ]]; then
        echo "0 0"
        return
    fi

    local avg_cpu avg_rss_kb
    avg_cpu=$(echo "scale=2; $total_cpu / $samples" | bc | fix_leading_zero)
    avg_rss_kb=$(echo "scale=0; $total_rss / $samples" | bc)

    echo "$avg_cpu $avg_rss_kb"
}

# bc omits leading zeros (e.g. ".13" instead of "0.13"); fix that
fix_leading_zero() {
    sed -e 's/^\./0./' -e 's/^-\./-0./'
}

format_rss() {
    local kb=$1
    echo "scale=1; $kb / 1024" | bc | fix_leading_zero
}

pct_diff() {
    local val=$1
    local ref=$2
    if [[ "$ref" == "0" || -z "$ref" ]]; then
        echo "N/A"
        return
    fi
    # Use scale=6 for intermediate precision, then round to 1 decimal
    echo "scale=6; x = ($val - $ref) / $ref * 100; scale=1; x / 1" | bc | fix_leading_zero
}

sign_pct() {
    local v=$1
    if [[ "$v" == "N/A" ]]; then
        echo "N/A"
    elif echo "$v >= 0" | bc -l | grep -q '^1$'; then
        echo "+${v}%"
    else
        echo "${v}%"
    fi
}

main() {
    check_prerequisites
    build_ttop

    echo "" >&2
    echo "=== Performance Benchmark ===" >&2
    echo "Duration: ${DURATION}s per tool, ${WARMUP}s warm-up, ${SAMPLE_INTERVAL}s sampling" >&2
    echo "" >&2

    read -r ttop_cpu ttop_rss <<< "$(benchmark_tool ttop "$TTOP_BIN")"
    echo "" >&2
    read -r top_cpu top_rss   <<< "$(benchmark_tool top top -d 1)"
    echo "" >&2
    read -r htop_cpu htop_rss <<< "$(benchmark_tool htop htop -d 10)"
    echo "" >&2

    local ttop_mb top_mb htop_mb
    ttop_mb=$(format_rss "$ttop_rss")
    top_mb=$(format_rss "$top_rss")
    htop_mb=$(format_rss "$htop_rss")

    local cpu_diff_top cpu_diff_htop mem_diff_top mem_diff_htop
    cpu_diff_top=$(pct_diff "$ttop_cpu" "$top_cpu")
    cpu_diff_htop=$(pct_diff "$ttop_cpu" "$htop_cpu")
    mem_diff_top=$(pct_diff "$ttop_rss" "$top_rss")
    mem_diff_htop=$(pct_diff "$ttop_rss" "$htop_rss")

    cpu_diff_top=$(sign_pct "$cpu_diff_top")
    cpu_diff_htop=$(sign_pct "$cpu_diff_htop")
    mem_diff_top=$(sign_pct "$mem_diff_top")
    mem_diff_htop=$(sign_pct "$mem_diff_htop")

    local table
    table=$(cat <<EOF
## Results

| Tool | Avg CPU (%) | Avg Memory (MB) |
|------|-------------|-----------------|
| ttop | ${ttop_cpu} | ${ttop_mb} |
| top  | ${top_cpu} | ${top_mb} |
| htop | ${htop_cpu} | ${htop_mb} |

## Comparison (ttop vs others)

| Metric | ttop vs top | ttop vs htop |
|--------|-------------|--------------|
| CPU    | ${cpu_diff_top} | ${cpu_diff_htop} |
| Memory | ${mem_diff_top} | ${mem_diff_htop} |
EOF
)

    echo "$table"
    echo "$table" > "$RESULTS_FILE"

    echo "" >&2
    echo "Results saved to $RESULTS_FILE" >&2
}

main "$@"
