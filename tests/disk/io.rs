use ttop::disk::io::{format_rate, parse_diskstats, DiskIoState};
use ttop::ui::{sparkline_char_scaled, SPARKLINE_CHARS};

// --- parse_diskstats ---

#[test]
fn parse_diskstats_returns_entries() {
    let entries = parse_diskstats();
    assert!(
        !entries.is_empty(),
        "should find at least one whole-disk device"
    );
}

#[test]
fn parse_diskstats_excludes_partitions() {
    let entries = parse_diskstats();
    for e in &entries {
        assert!(
            !e.device.ends_with(|c: char| c.is_ascii_digit())
                || std::path::Path::new(&format!("/sys/block/{}", e.device)).exists(),
            "device {} should be a whole disk (present in /sys/block/)",
            e.device
        );
    }
}

#[test]
fn parse_diskstats_excludes_virtual_devices() {
    let entries = parse_diskstats();
    let virtual_prefixes = ["loop", "ram", "dm-", "sr", "fd"];
    for e in &entries {
        assert!(
            !virtual_prefixes.iter().any(|p| e.device.starts_with(p)),
            "virtual device {} should be filtered out",
            e.device
        );
    }
}

#[test]
fn parse_diskstats_entries_have_nonempty_device() {
    let entries = parse_diskstats();
    for e in &entries {
        assert!(!e.device.is_empty());
    }
}

// --- format_rate ---

#[test]
fn format_rate_zero() {
    let text = format_rate(0.0);
    assert_eq!(text, "0B/s");
}

#[test]
fn format_rate_bytes_range() {
    let text = format_rate(500.0);
    assert_eq!(text, "500B/s");
}

#[test]
fn format_rate_kilobytes() {
    let text = format_rate(1024.0);
    assert_eq!(text, "1.0KB/s");
}

#[test]
fn format_rate_megabytes() {
    let text = format_rate(1024.0 * 1024.0);
    assert_eq!(text, "1.0MB/s");
}

#[test]
fn format_rate_gigabytes() {
    let text = format_rate(1024.0 * 1024.0 * 1024.0);
    assert_eq!(text, "1.0GB/s");
}

#[test]
fn format_rate_fractional_megabytes() {
    let text = format_rate(45.2 * 1024.0 * 1024.0);
    assert_eq!(text, "45.2MB/s");
}

// --- DiskIoState ---

#[test]
fn disk_io_state_new_does_not_panic() {
    let _state = DiskIoState::new();
}

#[test]
fn disk_io_state_default_does_not_panic() {
    let _state = DiskIoState::default();
}

#[test]
fn disk_io_state_discovers_devices() {
    let state = DiskIoState::new();
    assert!(state.device_count() > 0, "should find at least one device");
}

#[test]
fn disk_io_state_starts_with_empty_histories() {
    let state = DiskIoState::new();
    for h in &state.read_histories {
        assert!(h.is_empty());
    }
    for h in &state.write_histories {
        assert!(h.is_empty());
    }
}

#[test]
fn disk_io_state_first_update_no_data() {
    let mut state = DiskIoState::new();
    state.update(60);
    for h in &state.read_histories {
        assert!(h.is_empty(), "first update should not produce data (no prev snapshot)");
    }
}

#[test]
fn disk_io_state_second_update_produces_data() {
    let mut state = DiskIoState::new();
    state.update(60);
    state.update(60);
    for h in &state.read_histories {
        assert_eq!(h.len(), 1, "second update should produce one data point");
    }
    for h in &state.write_histories {
        assert_eq!(h.len(), 1);
    }
}

#[test]
fn disk_io_state_values_non_negative() {
    let mut state = DiskIoState::new();
    state.update(60);
    state.update(60);
    for h in &state.read_histories {
        if let Some(&val) = h.back() {
            assert!(val >= 0.0, "read bytes/sec should be non-negative: {val}");
        }
    }
    for h in &state.write_histories {
        if let Some(&val) = h.back() {
            assert!(val >= 0.0, "write bytes/sec should be non-negative: {val}");
        }
    }
}

#[test]
fn disk_io_state_trims_to_chart_width() {
    let mut state = DiskIoState::new();
    let width = 3;
    state.update(width);
    for _ in 0..10 {
        state.update(width);
    }
    for h in &state.read_histories {
        assert!(h.len() <= width, "read history {} exceeds width {}", h.len(), width);
    }
    for h in &state.write_histories {
        assert!(h.len() <= width, "write history {} exceeds width {}", h.len(), width);
    }
}

#[test]
fn disk_io_state_label_width_at_least_four() {
    let state = DiskIoState::new();
    assert!(state.label_width() >= 4);
}

#[test]
fn disk_io_state_rate_width_is_fixed() {
    let state = DiskIoState::new();
    assert_eq!(state.rate_width(), 10);
}

#[test]
fn format_rate_fits_within_rate_width() {
    let state = DiskIoState::new();
    let rw = state.rate_width();

    let test_values: &[f64] = &[
        0.0,
        1.0,
        512.0,
        1023.0,
        1024.0,
        10240.0,
        500.0 * 1024.0,
        1023.9 * 1024.0,
        1024.0 * 1024.0,
        45.2 * 1024.0 * 1024.0,
        500.0 * 1024.0 * 1024.0,
        1023.9 * 1024.0 * 1024.0,
        1024.0 * 1024.0 * 1024.0,
        10.0 * 1024.0 * 1024.0 * 1024.0,
    ];
    for &v in test_values {
        let text = format_rate(v);
        assert!(
            text.len() <= rw,
            "format_rate({v}) = \"{text}\" ({} chars) exceeds rate_width {rw}",
            text.len()
        );
    }
}

// --- sparkline_char_scaled ---

#[test]
fn sparkline_char_scaled_zero_max_returns_lowest() {
    assert_eq!(sparkline_char_scaled(0.0, 0.0), SPARKLINE_CHARS[0]);
    assert_eq!(sparkline_char_scaled(100.0, 0.0), SPARKLINE_CHARS[0]);
}

#[test]
fn sparkline_char_scaled_zero_value_returns_lowest() {
    assert_eq!(sparkline_char_scaled(0.0, 1000.0), SPARKLINE_CHARS[0]);
}

#[test]
fn sparkline_char_scaled_max_value_returns_highest() {
    assert_eq!(sparkline_char_scaled(1000.0, 1000.0), SPARKLINE_CHARS[7]);
}

#[test]
fn sparkline_char_scaled_half_value() {
    let ch = sparkline_char_scaled(500.0, 1000.0);
    assert!(
        SPARKLINE_CHARS.contains(&ch),
        "should return a valid sparkline char"
    );
}

#[test]
fn sparkline_char_scaled_over_max_clamps() {
    assert_eq!(sparkline_char_scaled(2000.0, 1000.0), SPARKLINE_CHARS[7]);
}

#[test]
fn sparkline_char_scaled_negative_value_clamps() {
    assert_eq!(sparkline_char_scaled(-10.0, 1000.0), SPARKLINE_CHARS[0]);
}
