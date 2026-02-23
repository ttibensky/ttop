use ttop::memory::usage::{
    format_human_bytes, format_mem_pair, parse_meminfo, ram_usage_pct, swap_usage_pct, MemInfo,
    MemState,
};

// --- parse_meminfo ---

#[test]
fn parse_meminfo_returns_nonzero_mem_total() {
    let info = parse_meminfo();
    assert!(info.mem_total_kb > 0, "MemTotal should be nonzero");
}

#[test]
fn parse_meminfo_mem_available_le_total() {
    let info = parse_meminfo();
    assert!(
        info.mem_available_kb <= info.mem_total_kb,
        "MemAvailable ({}) should not exceed MemTotal ({})",
        info.mem_available_kb,
        info.mem_total_kb
    );
}

#[test]
fn parse_meminfo_swap_free_le_total() {
    let info = parse_meminfo();
    assert!(
        info.swap_free_kb <= info.swap_total_kb || info.swap_total_kb == 0,
        "SwapFree ({}) should not exceed SwapTotal ({})",
        info.swap_free_kb,
        info.swap_total_kb
    );
}

// --- usage percentages ---

#[test]
fn ram_usage_pct_in_valid_range() {
    let info = parse_meminfo();
    let pct = ram_usage_pct(&info);
    assert!(
        (0.0..=100.0).contains(&pct),
        "RAM usage {} out of range",
        pct
    );
}

#[test]
fn swap_usage_pct_in_valid_range() {
    let info = parse_meminfo();
    let pct = swap_usage_pct(&info);
    assert!(
        (0.0..=100.0).contains(&pct),
        "Swap usage {} out of range",
        pct
    );
}

#[test]
fn ram_usage_pct_zero_total_returns_zero() {
    let info = MemInfo {
        mem_total_kb: 0,
        mem_available_kb: 0,
        swap_total_kb: 0,
        swap_free_kb: 0,
    };
    assert_eq!(ram_usage_pct(&info), 0.0);
}

#[test]
fn swap_usage_pct_zero_total_returns_zero() {
    let info = MemInfo {
        mem_total_kb: 16_000_000,
        mem_available_kb: 8_000_000,
        swap_total_kb: 0,
        swap_free_kb: 0,
    };
    assert_eq!(swap_usage_pct(&info), 0.0);
}

#[test]
fn ram_usage_pct_fully_used() {
    let info = MemInfo {
        mem_total_kb: 16_000_000,
        mem_available_kb: 0,
        swap_total_kb: 0,
        swap_free_kb: 0,
    };
    assert!((ram_usage_pct(&info) - 100.0).abs() < 0.01);
}

#[test]
fn ram_usage_pct_half_used() {
    let info = MemInfo {
        mem_total_kb: 16_000_000,
        mem_available_kb: 8_000_000,
        swap_total_kb: 0,
        swap_free_kb: 0,
    };
    assert!((ram_usage_pct(&info) - 50.0).abs() < 0.01);
}

#[test]
fn swap_usage_pct_half_used() {
    let info = MemInfo {
        mem_total_kb: 16_000_000,
        mem_available_kb: 8_000_000,
        swap_total_kb: 8_000_000,
        swap_free_kb: 4_000_000,
    };
    assert!((swap_usage_pct(&info) - 50.0).abs() < 0.01);
}

// --- format_human_bytes ---

#[test]
fn format_human_bytes_kilobytes() {
    assert_eq!(format_human_bytes(512), "512KB");
}

#[test]
fn format_human_bytes_megabytes() {
    assert_eq!(format_human_bytes(1500), "1.5MB");
}

#[test]
fn format_human_bytes_megabytes_exact() {
    assert_eq!(format_human_bytes(1024), "1.0MB");
}

#[test]
fn format_human_bytes_gigabytes() {
    assert_eq!(format_human_bytes(16_777_216), "16.0GB");
}

#[test]
fn format_human_bytes_terabytes() {
    let one_tb_in_kb = 1024 * 1024 * 1024;
    assert_eq!(format_human_bytes(one_tb_in_kb), "1.0TB");
}

#[test]
fn format_human_bytes_zero() {
    assert_eq!(format_human_bytes(0), "0KB");
}

// --- format_mem_pair ---

#[test]
fn format_mem_pair_gigabyte_range() {
    let total = 16_777_216; // 16 GiB in kB
    let used = total / 2;
    let text = format_mem_pair(used, total);
    assert!(text.ends_with("GB"), "should use GB unit, got {}", text);
    assert!(text.contains('/'), "should have slash separator, got {}", text);
}

#[test]
fn format_mem_pair_zero_total() {
    assert_eq!(format_mem_pair(0, 0), "0.0GB/0.0GB");
}

#[test]
fn format_mem_pair_zero_used() {
    let total = 8_388_608; // 8 GiB in kB
    let text = format_mem_pair(0, total);
    assert!(text.starts_with("0KB/") || text.starts_with("0.0"), "got {}", text);
    assert_eq!(text, "0KB/8.0GB");
}

#[test]
fn format_mem_pair_mixed_units() {
    let used_kb = 8_388_608; // 8 GiB
    let total_kb = 1024 * 1024 * 1024; // 1 TiB
    let text = format_mem_pair(used_kb, total_kb);
    assert_eq!(text, "8.0GB/1.0TB");
}

// --- MemState lifecycle ---

#[test]
fn mem_state_new_does_not_panic() {
    let _state = MemState::new();
}

#[test]
fn mem_state_starts_with_empty_histories() {
    let state = MemState::new();
    assert!(state.ram_history.is_empty());
    assert!(state.swap_history.is_empty());
}

#[test]
fn mem_state_update_adds_one_sample() {
    let mut state = MemState::new();
    state.update(60);
    assert_eq!(state.ram_history.len(), 1);
    assert_eq!(state.swap_history.len(), 1);
}

#[test]
fn mem_state_update_values_in_range() {
    let mut state = MemState::new();
    state.update(60);
    let ram = *state.ram_history.back().unwrap();
    let swap = *state.swap_history.back().unwrap();
    assert!((0.0..=100.0).contains(&ram), "RAM {} out of range", ram);
    assert!((0.0..=100.0).contains(&swap), "Swap {} out of range", swap);
}

#[test]
fn mem_state_update_trims_to_chart_width() {
    let mut state = MemState::new();
    let width = 5;
    for _ in 0..10 {
        state.update(width);
    }
    assert!(
        state.ram_history.len() <= width,
        "RAM history {} exceeds width {}",
        state.ram_history.len(),
        width
    );
    assert!(
        state.swap_history.len() <= width,
        "Swap history {} exceeds width {}",
        state.swap_history.len(),
        width
    );
}

#[test]
fn mem_state_current_reflects_latest_read() {
    let mut state = MemState::new();
    state.update(60);
    assert!(state.current.mem_total_kb > 0);
}

#[test]
fn mem_state_swap_available_reflects_system() {
    let state = MemState::new();
    let info = parse_meminfo();
    assert_eq!(state.swap_available(), info.swap_total_kb > 0);
}
