use ttop::cpu::temperature::{celsius_to_fahrenheit, discover_sensors, TempState};

#[test]
fn celsius_to_fahrenheit_freezing() {
    assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 0.01);
}

#[test]
fn celsius_to_fahrenheit_boiling() {
    assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < 0.01);
}

#[test]
fn celsius_to_fahrenheit_body_temp() {
    assert!((celsius_to_fahrenheit(37.0) - 98.6).abs() < 0.01);
}

#[test]
fn celsius_to_fahrenheit_typical_cpu_temp() {
    let f = celsius_to_fahrenheit(46.0);
    assert!((f - 114.8).abs() < 0.1);
}

#[test]
fn temp_state_new_does_not_panic() {
    let _state = TempState::new();
}

#[test]
fn temp_state_histories_match_sensor_count() {
    let state = TempState::new();
    assert_eq!(state.histories.len(), state.sensor_count());
}

#[test]
fn temp_state_starts_with_empty_histories() {
    let state = TempState::new();
    for h in &state.histories {
        assert!(h.is_empty());
    }
}

#[test]
fn temp_state_update_does_not_panic() {
    let mut state = TempState::new();
    state.update(60);
}

#[test]
fn temp_state_update_adds_samples_if_sensors_exist() {
    let mut state = TempState::new();
    state.update(60);
    if state.available() {
        for (i, h) in state.histories.iter().enumerate() {
            assert_eq!(h.len(), 1, "sensor {} should have 1 sample", i);
        }
    }
}

#[test]
fn temp_state_update_trims_to_chart_width() {
    let mut state = TempState::new();
    if !state.available() {
        return;
    }
    let width = 3;
    for _ in 0..10 {
        state.update(width);
    }
    for (i, h) in state.histories.iter().enumerate() {
        assert!(
            h.len() <= width,
            "sensor {} history len {} exceeds width {}",
            i,
            h.len(),
            width
        );
    }
}

#[test]
fn temp_state_values_in_sane_range() {
    let mut state = TempState::new();
    if !state.available() {
        return;
    }
    state.update(60);
    for (i, h) in state.histories.iter().enumerate() {
        if let Some(&val) = h.back() {
            assert!(
                (0.0..=150.0).contains(&val),
                "sensor {} temp {} out of sane range",
                i,
                val
            );
        }
    }
}

#[test]
fn temp_state_labels_match_sensor_count() {
    let state = TempState::new();
    assert_eq!(state.labels().len(), state.sensor_count());
}

#[test]
fn discover_sensors_does_not_panic() {
    let _sensors = discover_sensors();
}
