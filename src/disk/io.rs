use std::collections::VecDeque;
use std::fs;
use std::path::Path;

pub struct DiskIoEntry {
    pub device: String,
    pub read_sectors: u64,
    pub write_sectors: u64,
}

const VIRTUAL_DEVICE_PREFIXES: &[&str] = &["loop", "ram", "dm-", "sr", "fd"];

fn is_physical_disk(device: &str) -> bool {
    if VIRTUAL_DEVICE_PREFIXES.iter().any(|p| device.starts_with(p)) {
        return false;
    }
    Path::new(&format!("/sys/block/{device}")).exists()
}

pub fn parse_diskstats() -> Vec<DiskIoEntry> {
    let content = match fs::read_to_string("/proc/diskstats") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut entries = Vec::new();

    for line in content.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 14 {
            continue;
        }

        let device = fields[2];

        if !is_physical_disk(device) {
            continue;
        }

        let read_sectors: u64 = match fields[5].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };
        let write_sectors: u64 = match fields[9].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        entries.push(DiskIoEntry {
            device: device.to_string(),
            read_sectors,
            write_sectors,
        });
    }

    entries
}

const SECTOR_SIZE: u64 = 512;

pub fn format_rate(bytes_per_sec: f64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    const KB: f64 = 1024.0;

    if bytes_per_sec >= GB {
        format!("{:.1}GB/s", bytes_per_sec / GB)
    } else if bytes_per_sec >= MB {
        format!("{:.1}MB/s", bytes_per_sec / MB)
    } else if bytes_per_sec >= KB {
        format!("{:.1}KB/s", bytes_per_sec / KB)
    } else {
        format!("{:.0}B/s", bytes_per_sec)
    }
}

pub struct DiskIoState {
    pub devices: Vec<String>,
    prev_read_sectors: Vec<u64>,
    prev_write_sectors: Vec<u64>,
    pub read_histories: Vec<VecDeque<f64>>,
    pub write_histories: Vec<VecDeque<f64>>,
    pub max_observed: Vec<f64>,
    has_prev: bool,
}

impl Default for DiskIoState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskIoState {
    pub fn new() -> Self {
        let snapshot = parse_diskstats();
        let devices: Vec<String> = snapshot.iter().map(|e| e.device.clone()).collect();
        let count = devices.len();
        let prev_read: Vec<u64> = snapshot.iter().map(|e| e.read_sectors).collect();
        let prev_write: Vec<u64> = snapshot.iter().map(|e| e.write_sectors).collect();

        Self {
            devices,
            prev_read_sectors: prev_read,
            prev_write_sectors: prev_write,
            read_histories: vec![VecDeque::new(); count],
            write_histories: vec![VecDeque::new(); count],
            max_observed: vec![0.0; count],
            has_prev: false,
        }
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    pub fn update(&mut self, chart_width: usize) {
        let snapshot = parse_diskstats();

        for (i, device) in self.devices.iter().enumerate() {
            let entry = snapshot.iter().find(|e| &e.device == device);

            if let Some(entry) = entry {
                if self.has_prev {
                    let read_delta = entry.read_sectors.saturating_sub(self.prev_read_sectors[i]);
                    let write_delta = entry.write_sectors.saturating_sub(self.prev_write_sectors[i]);

                    let read_bps = (read_delta * SECTOR_SIZE) as f64;
                    let write_bps = (write_delta * SECTOR_SIZE) as f64;

                    self.read_histories[i].push_back(read_bps);
                    self.write_histories[i].push_back(write_bps);

                    let local_max = read_bps.max(write_bps);
                    if local_max > self.max_observed[i] {
                        self.max_observed[i] = local_max;
                    }
                }

                self.prev_read_sectors[i] = entry.read_sectors;
                self.prev_write_sectors[i] = entry.write_sectors;
            }

            while self.read_histories[i].len() > chart_width {
                self.read_histories[i].pop_front();
            }
            while self.write_histories[i].len() > chart_width {
                self.write_histories[i].pop_front();
            }
        }

        self.has_prev = true;
    }

    pub fn label_width(&self) -> usize {
        self.devices
            .iter()
            .map(|d| d.len() + 1) // +1 for the R/W suffix
            .max()
            .unwrap_or(4)
            .max(4)
    }

    pub fn rate_width(&self) -> usize {
        let mut max_w = 6; // minimum "0B/s" width
        for i in 0..self.devices.len() {
            if let Some(&v) = self.read_histories[i].back() {
                max_w = max_w.max(format_rate(v).len());
            }
            if let Some(&v) = self.write_histories[i].back() {
                max_w = max_w.max(format_rate(v).len());
            }
        }
        max_w
    }
}
