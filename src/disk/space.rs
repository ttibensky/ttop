use std::collections::VecDeque;
use std::ffi::CString;
use std::fs;

use crate::memory::usage::{format_mem_pair, max_mem_pair_width};

const VIRTUAL_FS_TYPES: &[&str] = &[
    "autofs",
    "bpf",
    "cgroup",
    "cgroup2",
    "configfs",
    "debugfs",
    "devpts",
    "devtmpfs",
    "efivarfs",
    "fuse.gvfsd-fuse",
    "fuse.portal",
    "fuse.snapfuse",
    "fusectl",
    "hugetlbfs",
    "mqueue",
    "nsfs",
    "overlay",
    "proc",
    "pstore",
    "ramfs",
    "securityfs",
    "squashfs",
    "sysfs",
    "tmpfs",
    "tracefs",
];

const PHYSICAL_DEVICE_PREFIXES: &[&str] = &[
    "/dev/sd",
    "/dev/nvme",
    "/dev/vd",
    "/dev/hd",
    "/dev/xvd",
    "/dev/mmcblk",
];

pub struct MountEntry {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
}

pub fn parse_proc_mounts() -> Vec<MountEntry> {
    let content = match fs::read_to_string("/proc/mounts") {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut seen_mounts = Vec::new();
    let mut entries = Vec::new();

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        if VIRTUAL_FS_TYPES.contains(&fs_type) {
            continue;
        }

        if !PHYSICAL_DEVICE_PREFIXES.iter().any(|p| device.starts_with(p)) {
            continue;
        }

        if seen_mounts.contains(&mount_point.to_string()) {
            continue;
        }

        seen_mounts.push(mount_point.to_string());
        entries.push(MountEntry {
            device: device.to_string(),
            mount_point: mount_point.to_string(),
            fs_type: fs_type.to_string(),
        });
    }

    entries
}

pub fn read_statvfs(path: &str) -> Option<(u64, u64)> {
    let c_path = CString::new(path).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };
    if ret != 0 {
        return None;
    }

    let frsize = stat.f_frsize;
    let total_bytes = stat.f_blocks * frsize;
    let free_bytes = stat.f_bfree * frsize;
    let used_bytes = total_bytes.saturating_sub(free_bytes);

    let total_kb = total_bytes / 1024;
    let used_kb = used_bytes / 1024;

    if total_kb == 0 {
        return None;
    }

    Some((used_kb, total_kb))
}

pub struct DiskSpaceState {
    mounts: Vec<MountEntry>,
    pub histories: Vec<VecDeque<f64>>,
    pub current_used_kb: Vec<u64>,
    pub current_total_kb: Vec<u64>,
}

impl Default for DiskSpaceState {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskSpaceState {
    pub fn new() -> Self {
        let mounts: Vec<MountEntry> = parse_proc_mounts()
            .into_iter()
            .filter(|m| read_statvfs(&m.mount_point).is_some())
            .collect();
        let count = mounts.len();
        Self {
            mounts,
            histories: vec![VecDeque::new(); count],
            current_used_kb: vec![0; count],
            current_total_kb: vec![0; count],
        }
    }

    pub fn mount_count(&self) -> usize {
        self.mounts.len()
    }

    pub fn labels(&self) -> Vec<&str> {
        self.mounts.iter().map(|m| m.mount_point.as_str()).collect()
    }

    pub fn update(&mut self, chart_width: usize) {
        for (i, mount) in self.mounts.iter().enumerate() {
            if let Some((used_kb, total_kb)) = read_statvfs(&mount.mount_point) {
                self.current_used_kb[i] = used_kb;
                self.current_total_kb[i] = total_kb;

                let pct = used_kb as f64 / total_kb as f64 * 100.0;
                self.histories[i].push_back(pct);
            }

            while self.histories[i].len() > chart_width {
                self.histories[i].pop_front();
            }
        }
    }

    pub fn abs_text(&self, index: usize) -> String {
        format_mem_pair(self.current_used_kb[index], self.current_total_kb[index])
    }

    pub fn abs_width(&self) -> usize {
        self.current_total_kb
            .iter()
            .map(|&total| max_mem_pair_width(total))
            .max()
            .unwrap_or(0)
    }

    pub fn label_width(&self) -> usize {
        self.mounts
            .iter()
            .map(|m| m.mount_point.len())
            .max()
            .unwrap_or(1)
            .max(1)
    }
}

