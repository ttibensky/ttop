use std::fs;
use std::path::{Path, PathBuf};

pub struct AmdGpu {
    pub name: String,
    pub card_path: PathBuf,
    pub hwmon_path: Option<PathBuf>,
}

pub fn detect() -> Option<AmdGpu> {
    let drm_base = PathBuf::from("/sys/class/drm");
    let entries = fs::read_dir(&drm_base).ok()?;

    for entry in entries.flatten() {
        let dir_name = entry.file_name().to_string_lossy().to_string();
        if !dir_name.starts_with("card") || dir_name.contains('-') {
            continue;
        }

        let device_path = entry.path().join("device");
        let vendor = match fs::read_to_string(device_path.join("vendor")) {
            Ok(v) => v.trim().to_string(),
            Err(_) => continue,
        };

        if vendor != "0x1002" {
            continue;
        }

        let name = fs::read_to_string(device_path.join("product_name"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "AMD GPU".to_string());

        let hwmon_path = find_hwmon(&device_path);

        return Some(AmdGpu {
            name,
            card_path: entry.path(),
            hwmon_path,
        });
    }

    None
}

fn find_hwmon(device_path: &Path) -> Option<PathBuf> {
    let entries = fs::read_dir(device_path.join("hwmon")).ok()?;

    for entry in entries.flatten() {
        let name_path = entry.path().join("name");
        if let Ok(name) = fs::read_to_string(&name_path)
            && name.trim() == "amdgpu"
        {
            return Some(entry.path());
        }
    }

    None
}

pub fn read_utilization(card_path: &Path) -> Option<f64> {
    fs::read_to_string(card_path.join("device/gpu_busy_percent"))
        .ok()
        .and_then(|s| s.trim().parse().ok())
}

pub fn read_memory(card_path: &Path) -> Option<(u64, u64)> {
    let used = fs::read_to_string(card_path.join("device/mem_info_vram_used"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())?;
    let total = fs::read_to_string(card_path.join("device/mem_info_vram_total"))
        .ok()
        .and_then(|s| s.trim().parse::<u64>().ok())?;
    Some((used, total))
}

pub fn read_temperature(hwmon_path: &Path) -> Option<f64> {
    fs::read_to_string(hwmon_path.join("temp1_input"))
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .map(|millideg| millideg as f64 / 1000.0)
}
