use std::collections::VecDeque;
use std::fs;

pub struct MemInfo {
    pub mem_total_kb: u64,
    pub mem_available_kb: u64,
    pub swap_total_kb: u64,
    pub swap_free_kb: u64,
}

pub struct MemState {
    pub ram_history: VecDeque<f64>,
    pub swap_history: VecDeque<f64>,
    pub current: MemInfo,
}

pub fn parse_meminfo() -> MemInfo {
    let content = fs::read_to_string("/proc/meminfo").expect("failed to read /proc/meminfo");
    let mut mem_total_kb = 0u64;
    let mut mem_available_kb = 0u64;
    let mut swap_total_kb = 0u64;
    let mut swap_free_kb = 0u64;

    for line in content.lines() {
        let mut parts = line.split_whitespace();
        let key = match parts.next() {
            Some(k) => k,
            None => continue,
        };
        let value: u64 = match parts.next().and_then(|v| v.parse().ok()) {
            Some(v) => v,
            None => continue,
        };

        match key {
            "MemTotal:" => mem_total_kb = value,
            "MemAvailable:" => mem_available_kb = value,
            "SwapTotal:" => swap_total_kb = value,
            "SwapFree:" => swap_free_kb = value,
            _ => {}
        }
    }

    MemInfo {
        mem_total_kb,
        mem_available_kb,
        swap_total_kb,
        swap_free_kb,
    }
}

pub fn ram_usage_pct(info: &MemInfo) -> f64 {
    if info.mem_total_kb == 0 {
        return 0.0;
    }
    let used = info.mem_total_kb.saturating_sub(info.mem_available_kb);
    (used as f64 / info.mem_total_kb as f64) * 100.0
}

pub fn swap_usage_pct(info: &MemInfo) -> f64 {
    if info.swap_total_kb == 0 {
        return 0.0;
    }
    let used = info.swap_total_kb.saturating_sub(info.swap_free_kb);
    (used as f64 / info.swap_total_kb as f64) * 100.0
}

/// Format a kB value as a human-readable string with one decimal place and
/// full unit suffix (e.g. `5.6GB`, `512.3MB`).
pub fn format_human_bytes(kb: u64) -> String {
    let bytes = kb as f64 * 1024.0;
    const TB: f64 = 1024.0 * 1024.0 * 1024.0 * 1024.0;
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;

    if bytes >= TB {
        format!("{:.1}TB", bytes / TB)
    } else if bytes >= GB {
        format!("{:.1}GB", bytes / GB)
    } else if bytes >= MB {
        format!("{:.1}MB", bytes / MB)
    } else {
        format!("{:.0}KB", bytes / 1024.0)
    }
}

/// Format a used/total pair where each value carries its own unit
/// (e.g. `5.6GB/16.0GB`, `512.3MB/2.0TB`).
/// Returns `0.0GB/0.0GB` when total is zero (e.g. swap disabled).
pub fn format_mem_pair(used_kb: u64, total_kb: u64) -> String {
    if total_kb == 0 {
        return "0.0GB/0.0GB".to_string();
    }
    format!("{}/{}", format_human_bytes(used_kb), format_human_bytes(total_kb))
}

impl Default for MemState {
    fn default() -> Self {
        Self::new()
    }
}

impl MemState {
    pub fn new() -> Self {
        let current = parse_meminfo();
        Self {
            ram_history: VecDeque::new(),
            swap_history: VecDeque::new(),
            current,
        }
    }

    pub fn update(&mut self, chart_width: usize) {
        self.current = parse_meminfo();

        let ram_pct = ram_usage_pct(&self.current);
        self.ram_history.push_back(ram_pct);
        while self.ram_history.len() > chart_width {
            self.ram_history.pop_front();
        }

        let swap_pct = swap_usage_pct(&self.current);
        self.swap_history.push_back(swap_pct);
        while self.swap_history.len() > chart_width {
            self.swap_history.pop_front();
        }
    }

    pub fn swap_available(&self) -> bool {
        self.current.swap_total_kb > 0
    }
}
