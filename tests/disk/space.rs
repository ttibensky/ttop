use ttop::disk::space::{parse_proc_mounts, read_statvfs, DiskSpaceState};
use ttop::memory::usage::{format_mem_pair, max_mem_pair_width};

// --- parse_proc_mounts ---

#[test]
fn parse_proc_mounts_returns_at_least_root() {
    let mounts = parse_proc_mounts();
    let has_root = mounts.iter().any(|m| m.mount_point == "/");
    assert!(has_root, "root filesystem should always be present");
}

#[test]
fn parse_proc_mounts_filters_virtual_filesystems() {
    let mounts = parse_proc_mounts();
    for m in &mounts {
        assert_ne!(m.fs_type, "sysfs", "sysfs should be filtered out");
        assert_ne!(m.fs_type, "proc", "proc should be filtered out");
        assert_ne!(m.fs_type, "tmpfs", "tmpfs should be filtered out");
        assert_ne!(m.fs_type, "devtmpfs", "devtmpfs should be filtered out");
        assert_ne!(m.fs_type, "squashfs", "squashfs should be filtered out");
        assert_ne!(m.fs_type, "fuse.snapfuse", "fuse.snapfuse should be filtered out");
    }
}

#[test]
fn parse_proc_mounts_only_physical_devices() {
    let mounts = parse_proc_mounts();
    let physical_prefixes = ["/dev/sd", "/dev/nvme", "/dev/vd", "/dev/hd", "/dev/xvd", "/dev/mmcblk"];
    for m in &mounts {
        assert!(
            physical_prefixes.iter().any(|p| m.device.starts_with(p)),
            "device {} should have a physical device prefix",
            m.device
        );
    }
}

#[test]
fn parse_proc_mounts_no_duplicate_mount_points() {
    let mounts = parse_proc_mounts();
    let mut seen = Vec::new();
    for m in &mounts {
        assert!(
            !seen.contains(&m.mount_point),
            "duplicate mount point: {}",
            m.mount_point
        );
        seen.push(m.mount_point.clone());
    }
}

#[test]
fn parse_proc_mounts_entries_have_nonempty_fields() {
    let mounts = parse_proc_mounts();
    for m in &mounts {
        assert!(!m.device.is_empty(), "device should not be empty");
        assert!(!m.mount_point.is_empty(), "mount_point should not be empty");
        assert!(!m.fs_type.is_empty(), "fs_type should not be empty");
    }
}

// --- read_statvfs ---

#[test]
fn read_statvfs_root_returns_some() {
    let result = read_statvfs("/");
    assert!(result.is_some(), "statvfs on / should succeed");
}

#[test]
fn read_statvfs_root_total_nonzero() {
    let (used_kb, total_kb) = read_statvfs("/").unwrap();
    assert!(total_kb > 0, "root total should be nonzero");
    assert!(used_kb <= total_kb, "used ({used_kb}) should not exceed total ({total_kb})");
}

#[test]
fn read_statvfs_invalid_path_returns_none() {
    let result = read_statvfs("/nonexistent_path_12345");
    assert!(result.is_none(), "statvfs on invalid path should return None");
}

// --- DiskSpaceState ---

#[test]
fn disk_space_state_new_does_not_panic() {
    let _state = DiskSpaceState::new();
}

#[test]
fn disk_space_state_default_does_not_panic() {
    let _state = DiskSpaceState::default();
}

#[test]
fn disk_space_state_has_at_least_one_mount() {
    let state = DiskSpaceState::new();
    assert!(state.mount_count() > 0, "should discover at least root");
}

#[test]
fn disk_space_state_starts_with_empty_histories() {
    let state = DiskSpaceState::new();
    for h in &state.histories {
        assert!(h.is_empty());
    }
}

#[test]
fn disk_space_state_update_adds_one_sample() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    for h in &state.histories {
        assert_eq!(h.len(), 1);
    }
}

#[test]
fn disk_space_state_update_values_in_range() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    for h in &state.histories {
        let val = *h.back().unwrap();
        assert!(
            (0.0..=100.0).contains(&val),
            "disk usage {} out of range",
            val
        );
    }
}

#[test]
fn disk_space_state_update_trims_to_chart_width() {
    let mut state = DiskSpaceState::new();
    let width = 3;
    for _ in 0..10 {
        state.update(width);
    }
    for h in &state.histories {
        assert!(
            h.len() <= width,
            "history {} exceeds width {}",
            h.len(),
            width
        );
    }
}

#[test]
fn disk_space_state_labels_match_mount_count() {
    let state = DiskSpaceState::new();
    assert_eq!(state.labels().len(), state.mount_count());
}

#[test]
fn disk_space_state_abs_text_nonempty() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    for i in 0..state.mount_count() {
        let text = state.abs_text(i);
        assert!(!text.is_empty(), "abs_text should not be empty");
        assert!(text.contains('/'), "abs_text should contain slash: {text}");
    }
}

#[test]
fn disk_space_state_abs_width_covers_all_used_values() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    let fixed_w = state.abs_width();

    for i in 0..state.mount_count() {
        let total = state.current_total_kb[i];
        let test_values: &[u64] = &[0, 1023, 1024, 1_048_575, 1_048_576, total];
        for &used in test_values {
            if used > total {
                continue;
            }
            let text = format_mem_pair(used, total);
            assert!(
                text.len() <= fixed_w,
                "disk mount {i}: format_mem_pair({used}, {total}) = \"{text}\" ({} chars) exceeds fixed width {fixed_w}",
                text.len()
            );
        }
    }
}

#[test]
fn disk_space_state_abs_width_equals_max_mem_pair_width() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    let expected = state
        .current_total_kb
        .iter()
        .map(|&total| max_mem_pair_width(total))
        .max()
        .unwrap_or(0);
    assert_eq!(state.abs_width(), expected);
}

#[test]
fn disk_space_state_current_values_populated_after_update() {
    let mut state = DiskSpaceState::new();
    state.update(60);
    for i in 0..state.mount_count() {
        assert!(
            state.current_total_kb[i] > 0,
            "total_kb for mount {} should be nonzero",
            i
        );
    }
}
