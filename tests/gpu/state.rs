use ttop::gpu::GpuState;

#[test]
fn gpu_state_new_does_not_panic() {
    let _state = GpuState::new();
}

#[test]
fn gpu_state_default_does_not_panic() {
    let _state = GpuState::default();
}

#[test]
fn gpu_state_starts_with_empty_histories() {
    let state = GpuState::new();
    assert!(state.util_history.is_empty());
    assert!(state.mem_history.is_empty());
    assert!(state.temp_history.is_empty());
}

#[test]
fn gpu_state_starts_with_zero_memory() {
    let state = GpuState::new();
    assert_eq!(state.current_mem_used_kb, 0);
    assert_eq!(state.current_mem_total_kb, 0);
}

#[test]
fn gpu_state_update_does_not_panic() {
    let mut state = GpuState::new();
    state.update(60);
}

#[test]
fn gpu_state_update_adds_samples_if_available() {
    let mut state = GpuState::new();
    state.update(60);
    if state.available() {
        assert_eq!(state.util_history.len(), 1);
        assert_eq!(state.mem_history.len(), 1);
    }
}

#[test]
fn gpu_state_update_trims_to_chart_width() {
    let mut state = GpuState::new();
    if !state.available() {
        return;
    }
    let width = 3;
    for _ in 0..10 {
        state.update(width);
    }
    assert!(state.util_history.len() <= width);
    assert!(state.mem_history.len() <= width);
    assert!(state.temp_history.len() <= width);
}

#[test]
fn gpu_state_util_values_in_range() {
    let mut state = GpuState::new();
    if !state.available() {
        return;
    }
    state.update(60);
    if let Some(&val) = state.util_history.back() {
        assert!(
            (0.0..=100.0).contains(&val),
            "GPU utilization {} out of range",
            val
        );
    }
}

#[test]
fn gpu_state_mem_values_in_range() {
    let mut state = GpuState::new();
    if !state.available() {
        return;
    }
    state.update(60);
    if let Some(&val) = state.mem_history.back() {
        assert!(
            (0.0..=100.0).contains(&val),
            "GPU memory usage {} out of range",
            val
        );
    }
}

#[test]
fn gpu_state_temp_values_in_sane_range() {
    let mut state = GpuState::new();
    if !state.available() {
        return;
    }
    state.update(60);
    if let Some(&val) = state.temp_history.back() {
        assert!(
            (0.0..=150.0).contains(&val),
            "GPU temperature {} out of sane range",
            val
        );
    }
}

#[test]
fn gpu_state_name_nonempty_when_available() {
    let state = GpuState::new();
    if state.available() {
        assert!(!state.name.is_empty(), "GPU name should not be empty");
    }
}

#[test]
fn gpu_state_name_empty_when_unavailable() {
    let state = GpuState::new();
    if !state.available() {
        assert!(state.name.is_empty());
    }
}

#[test]
fn gpu_state_has_temperature_consistent() {
    let mut state = GpuState::new();
    if !state.available() {
        return;
    }
    state.update(60);
    if state.has_temperature() && !state.temp_history.is_empty() {
        assert!(state.temp_history.back().is_some());
    }
}
