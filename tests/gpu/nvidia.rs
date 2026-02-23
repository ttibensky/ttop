use ttop::gpu::nvidia::{detect, read_snapshot};

#[test]
fn detect_does_not_panic() {
    let _gpu = detect();
}

#[test]
fn detect_returns_name_if_present() {
    if let Some(gpu) = detect() {
        assert!(!gpu.name.is_empty(), "detected NVIDIA GPU should have a name");
    }
}

#[test]
fn read_snapshot_does_not_panic() {
    let _snapshot = read_snapshot();
}

#[test]
fn read_snapshot_values_in_range_if_available() {
    if let Some(snap) = read_snapshot() {
        assert!(
            (0.0..=100.0).contains(&snap.utilization_pct),
            "utilization {} out of range",
            snap.utilization_pct
        );
        assert!(
            snap.memory_used_mib <= snap.memory_total_mib,
            "memory used ({}) should not exceed total ({})",
            snap.memory_used_mib,
            snap.memory_total_mib
        );
        assert!(
            (0.0..=150.0).contains(&snap.temperature_c),
            "temperature {} out of sane range",
            snap.temperature_c
        );
    }
}

#[test]
fn read_snapshot_requires_detect() {
    if detect().is_none() {
        assert!(
            read_snapshot().is_none(),
            "read_snapshot should return None when no NVIDIA GPU detected"
        );
    }
}
