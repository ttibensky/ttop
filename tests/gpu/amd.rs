use ttop::gpu::amd::{detect, read_memory, read_temperature, read_utilization};

#[test]
fn detect_does_not_panic() {
    let _gpu = detect();
}

#[test]
fn detect_returns_name_if_present() {
    if let Some(gpu) = detect() {
        assert!(!gpu.name.is_empty(), "detected AMD GPU should have a name");
    }
}

#[test]
fn read_utilization_does_not_panic() {
    if let Some(gpu) = detect() {
        let _util = read_utilization(&gpu.card_path);
    }
}

#[test]
fn read_utilization_in_range_if_available() {
    if let Some(gpu) = detect()
        && let Some(util) = read_utilization(&gpu.card_path)
    {
        assert!(
            (0.0..=100.0).contains(&util),
            "AMD GPU utilization {} out of range",
            util
        );
    }
}

#[test]
fn read_memory_does_not_panic() {
    if let Some(gpu) = detect() {
        let _mem = read_memory(&gpu.card_path);
    }
}

#[test]
fn read_memory_used_le_total() {
    if let Some(gpu) = detect()
        && let Some((used, total)) = read_memory(&gpu.card_path)
    {
        assert!(
            used <= total,
            "VRAM used ({}) should not exceed total ({})",
            used,
            total
        );
    }
}

#[test]
fn read_temperature_does_not_panic() {
    if let Some(gpu) = detect()
        && let Some(hwmon) = &gpu.hwmon_path
    {
        let _temp = read_temperature(hwmon);
    }
}

#[test]
fn read_temperature_in_sane_range() {
    if let Some(gpu) = detect()
        && let Some(hwmon) = &gpu.hwmon_path
        && let Some(temp) = read_temperature(hwmon)
    {
        assert!(
            (0.0..=150.0).contains(&temp),
            "AMD GPU temperature {} out of sane range",
            temp
        );
    }
}
