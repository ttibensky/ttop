use ttop::memory::temperature::{discover_sensors, MemTempState};

#[test]
fn discover_sensors_does_not_panic() {
    let _sensors = discover_sensors();
}

#[test]
fn mem_temp_state_new_does_not_panic() {
    let _state = MemTempState::new();
}

#[test]
fn mem_temp_state_starts_with_empty_histories() {
    let state = MemTempState::new();
    for h in &state.histories {
        assert!(h.is_empty());
    }
}

#[test]
fn mem_temp_state_sensor_count_matches_histories() {
    let state = MemTempState::new();
    assert_eq!(state.sensor_count(), state.histories.len());
}

#[test]
fn mem_temp_state_available_consistent() {
    let state = MemTempState::new();
    assert_eq!(state.available(), state.sensor_count() > 0);
}

#[test]
fn mem_temp_state_labels_count_matches() {
    let state = MemTempState::new();
    assert_eq!(state.labels().len(), state.sensor_count());
}

#[test]
fn mem_temp_state_update_trims_to_chart_width() {
    let mut state = MemTempState::new();
    let width = 5;
    for _ in 0..10 {
        state.update(width);
    }
    for h in &state.histories {
        assert!(
            h.len() <= width,
            "history length {} exceeds chart width {}",
            h.len(),
            width
        );
    }
}

#[test]
fn mem_temp_state_values_in_sane_range() {
    let mut state = MemTempState::new();
    state.update(60);
    for h in &state.histories {
        if let Some(&temp) = h.back() {
            assert!(
                (-20.0..=150.0).contains(&temp),
                "DIMM temperature {} out of sane range",
                temp
            );
        }
    }
}
