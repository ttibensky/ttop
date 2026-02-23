use std::collections::VecDeque;
use std::fs;

#[derive(Clone)]
pub struct CpuTimes {
    pub user: u64,
    pub nice: u64,
    pub system: u64,
    pub idle: u64,
    pub iowait: u64,
    pub irq: u64,
    pub softirq: u64,
    pub steal: u64,
}

impl CpuTimes {
    pub fn total(&self) -> u64 {
        self.user + self.nice + self.system + self.idle + self.iowait + self.irq + self.softirq + self.steal
    }

    pub fn idle_total(&self) -> u64 {
        self.idle + self.iowait
    }
}

pub struct CpuState {
    prev_snapshot: Vec<CpuTimes>,
    pub histories: Vec<VecDeque<f64>>,
}

pub fn parse_proc_stat() -> Vec<CpuTimes> {
    let content = fs::read_to_string("/proc/stat").expect("failed to read /proc/stat");
    let mut cores = Vec::new();

    for line in content.lines() {
        if line.starts_with("cpu") && !line.starts_with("cpu ") {
            let fields: Vec<u64> = line
                .split_whitespace()
                .skip(1)
                .take(8)
                .map(|f| f.parse().unwrap_or(0))
                .collect();

            if fields.len() >= 8 {
                cores.push(CpuTimes {
                    user: fields[0],
                    nice: fields[1],
                    system: fields[2],
                    idle: fields[3],
                    iowait: fields[4],
                    irq: fields[5],
                    softirq: fields[6],
                    steal: fields[7],
                });
            }
        }
    }

    cores
}

pub fn compute_usage(prev: &CpuTimes, curr: &CpuTimes) -> f64 {
    let total_delta = curr.total().saturating_sub(prev.total());
    let idle_delta = curr.idle_total().saturating_sub(prev.idle_total());

    if total_delta == 0 {
        return 0.0;
    }

    ((total_delta - idle_delta) as f64 / total_delta as f64) * 100.0
}

impl Default for CpuState {
    fn default() -> Self {
        Self::new()
    }
}

impl CpuState {
    pub fn new() -> Self {
        let snapshot = parse_proc_stat();
        let core_count = snapshot.len();
        Self {
            prev_snapshot: snapshot,
            histories: vec![VecDeque::new(); core_count],
        }
    }

    pub fn update(&mut self, chart_width: usize) {
        let curr = parse_proc_stat();

        for (i, (prev, cur)) in self.prev_snapshot.iter().zip(curr.iter()).enumerate() {
            let usage = compute_usage(prev, cur);

            if i < self.histories.len() {
                self.histories[i].push_back(usage);
                while self.histories[i].len() > chart_width {
                    self.histories[i].pop_front();
                }
            }
        }

        self.prev_snapshot = curr;
    }

    pub fn core_count(&self) -> usize {
        self.histories.len()
    }
}
