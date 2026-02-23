use ttop::cpu::utilization::{compute_usage, parse_proc_stat, CpuState, CpuTimes};

fn make_times(user: u64, nice: u64, system: u64, idle: u64, iowait: u64) -> CpuTimes {
    CpuTimes {
        user,
        nice,
        system,
        idle,
        iowait,
        irq: 0,
        softirq: 0,
        steal: 0,
    }
}

#[test]
fn cpu_times_total_sums_all_fields() {
    let t = CpuTimes {
        user: 10,
        nice: 20,
        system: 30,
        idle: 40,
        iowait: 5,
        irq: 3,
        softirq: 2,
        steal: 1,
    };
    assert_eq!(t.total(), 111);
}

#[test]
fn cpu_times_idle_total_sums_idle_and_iowait() {
    let t = make_times(100, 0, 50, 800, 50);
    assert_eq!(t.idle_total(), 850);
}

#[test]
fn compute_usage_fully_idle() {
    let prev = make_times(0, 0, 0, 0, 0);
    let curr = make_times(0, 0, 0, 1000, 0);
    assert!((compute_usage(&prev, &curr) - 0.0).abs() < 0.01);
}

#[test]
fn compute_usage_fully_busy() {
    let prev = make_times(0, 0, 0, 0, 0);
    let curr = make_times(1000, 0, 0, 0, 0);
    assert!((compute_usage(&prev, &curr) - 100.0).abs() < 0.01);
}

#[test]
fn compute_usage_half_busy() {
    let prev = make_times(0, 0, 0, 0, 0);
    let curr = make_times(500, 0, 0, 500, 0);
    assert!((compute_usage(&prev, &curr) - 50.0).abs() < 0.01);
}

#[test]
fn compute_usage_zero_delta_returns_zero() {
    let prev = make_times(100, 0, 0, 100, 0);
    let curr = make_times(100, 0, 0, 100, 0);
    assert_eq!(compute_usage(&prev, &curr), 0.0);
}

#[test]
fn compute_usage_mixed_workload() {
    let prev = make_times(0, 0, 0, 0, 0);
    let curr = make_times(300, 0, 100, 500, 100);
    // total=1000, idle=600, busy=400
    assert!((compute_usage(&prev, &curr) - 40.0).abs() < 0.01);
}

#[test]
fn compute_usage_incremental_deltas() {
    let prev = make_times(1000, 0, 500, 8000, 500);
    let curr = make_times(1200, 0, 600, 8800, 600);
    // delta: user=200, system=100, idle=800, iowait=100 -> total=1200, idle=900, busy=300
    assert!((compute_usage(&prev, &curr) - 25.0).abs() < 0.01);
}

#[test]
fn parse_proc_stat_returns_cores() {
    let cores = parse_proc_stat();
    assert!(!cores.is_empty(), "should detect at least one core");
}

#[test]
fn parse_proc_stat_cores_have_nonzero_totals() {
    let cores = parse_proc_stat();
    for (i, core) in cores.iter().enumerate() {
        assert!(core.total() > 0, "core {} should have nonzero total time", i);
    }
}

#[test]
fn cpu_state_new_detects_cores() {
    let state = CpuState::new();
    assert!(state.core_count() > 0);
    assert_eq!(state.core_count(), state.histories.len());
}

#[test]
fn cpu_state_new_starts_with_empty_histories() {
    let state = CpuState::new();
    for h in &state.histories {
        assert!(h.is_empty());
    }
}

#[test]
fn cpu_state_update_adds_one_sample_per_core() {
    let mut state = CpuState::new();
    state.update(60);
    for (i, h) in state.histories.iter().enumerate() {
        assert_eq!(h.len(), 1, "core {} should have exactly 1 sample", i);
    }
}

#[test]
fn cpu_state_update_values_in_valid_range() {
    let mut state = CpuState::new();
    state.update(60);
    for (i, h) in state.histories.iter().enumerate() {
        let val = *h.back().unwrap();
        assert!((0.0..=100.0).contains(&val), "core {} usage {} out of range", i, val);
    }
}

#[test]
fn cpu_state_update_trims_to_chart_width() {
    let mut state = CpuState::new();
    let width = 5;
    for _ in 0..10 {
        state.update(width);
    }
    for (i, h) in state.histories.iter().enumerate() {
        assert!(
            h.len() <= width,
            "core {} history len {} exceeds width {}",
            i,
            h.len(),
            width
        );
    }
}

#[test]
fn cpu_state_core_count_matches_proc_stat() {
    let cores = parse_proc_stat();
    let state = CpuState::new();
    assert_eq!(state.core_count(), cores.len());
}
