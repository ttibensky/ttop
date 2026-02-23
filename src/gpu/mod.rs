pub mod amd;
pub mod nvidia;

use std::collections::VecDeque;
use std::path::PathBuf;

pub use amd::AmdGpu;
pub use nvidia::NvidiaGpu;

enum GpuBackend {
    Nvidia,
    Amd {
        card_path: PathBuf,
        hwmon_path: Option<PathBuf>,
    },
    None,
}

pub struct GpuState {
    backend: GpuBackend,
    pub name: String,
    pub util_history: VecDeque<f64>,
    pub mem_history: VecDeque<f64>,
    pub temp_history: VecDeque<f64>,
    pub current_mem_used_kb: u64,
    pub current_mem_total_kb: u64,
}

impl Default for GpuState {
    fn default() -> Self {
        Self::new()
    }
}

impl GpuState {
    pub fn new() -> Self {
        if let Some(gpu) = nvidia::detect() {
            return Self {
                name: gpu.name,
                backend: GpuBackend::Nvidia,
                util_history: VecDeque::new(),
                mem_history: VecDeque::new(),
                temp_history: VecDeque::new(),
                current_mem_used_kb: 0,
                current_mem_total_kb: 0,
            };
        }

        if let Some(gpu) = amd::detect() {
            let state = Self {
                name: gpu.name,
                backend: GpuBackend::Amd {
                    card_path: gpu.card_path,
                    hwmon_path: gpu.hwmon_path,
                },
                util_history: VecDeque::new(),
                mem_history: VecDeque::new(),
                temp_history: VecDeque::new(),
                current_mem_used_kb: 0,
                current_mem_total_kb: 0,
            };
            return state;
        }

        Self {
            name: String::new(),
            backend: GpuBackend::None,
            util_history: VecDeque::new(),
            mem_history: VecDeque::new(),
            temp_history: VecDeque::new(),
            current_mem_used_kb: 0,
            current_mem_total_kb: 0,
        }
    }

    pub fn available(&self) -> bool {
        !matches!(self.backend, GpuBackend::None)
    }

    pub fn update(&mut self, chart_width: usize) {
        let data = match &self.backend {
            GpuBackend::Nvidia => nvidia::read_snapshot().map(|s| {
                let mem_pct = if s.memory_total_mib > 0 {
                    s.memory_used_mib as f64 / s.memory_total_mib as f64 * 100.0
                } else {
                    0.0
                };
                (
                    s.utilization_pct,
                    mem_pct,
                    Some(s.temperature_c),
                    s.memory_used_mib * 1024,
                    s.memory_total_mib * 1024,
                )
            }),
            GpuBackend::Amd {
                card_path,
                hwmon_path,
            } => {
                let util = amd::read_utilization(card_path);
                let mem = amd::read_memory(card_path);
                let temp = hwmon_path.as_deref().and_then(amd::read_temperature);

                match (util, mem) {
                    (Some(u), Some((used, total))) => {
                        let mem_pct = if total > 0 {
                            used as f64 / total as f64 * 100.0
                        } else {
                            0.0
                        };
                        Some((u, mem_pct, temp, used / 1024, total / 1024))
                    }
                    _ => None,
                }
            }
            GpuBackend::None => None,
        };

        if let Some((util, mem_pct, temp, used_kb, total_kb)) = data {
            self.util_history.push_back(util);
            self.mem_history.push_back(mem_pct);
            if let Some(t) = temp {
                self.temp_history.push_back(t);
            }
            self.current_mem_used_kb = used_kb;
            self.current_mem_total_kb = total_kb;
        }

        self.trim_histories(chart_width);
    }

    fn trim_histories(&mut self, chart_width: usize) {
        while self.util_history.len() > chart_width {
            self.util_history.pop_front();
        }
        while self.mem_history.len() > chart_width {
            self.mem_history.pop_front();
        }
        while self.temp_history.len() > chart_width {
            self.temp_history.pop_front();
        }
    }

    pub fn has_temperature(&self) -> bool {
        !self.temp_history.is_empty()
            || match &self.backend {
                GpuBackend::Nvidia => true,
                GpuBackend::Amd { hwmon_path, .. } => hwmon_path.is_some(),
                GpuBackend::None => false,
            }
    }
}
