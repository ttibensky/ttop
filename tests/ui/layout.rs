use ttop::memory::usage::{format_mem_pair, max_mem_pair_width};
use ttop::ui::{
    core_columns, gpu_abs_width, label_width, mem_abs_width, mem_col_chart_width, temp_chart_width,
    util_chart_width,
};

#[test]
fn label_width_zero_cores() {
    assert_eq!(label_width(0), 2);
}

#[test]
fn label_width_single_core() {
    assert_eq!(label_width(1), 2);
}

#[test]
fn label_width_ten_cores() {
    assert_eq!(label_width(10), 2);
}

#[test]
fn label_width_eleven_cores() {
    assert_eq!(label_width(11), 3);
}

#[test]
fn label_width_hundred_cores() {
    assert_eq!(label_width(100), 3);
}

#[test]
fn label_width_hundred_one_cores() {
    assert_eq!(label_width(101), 4);
}

#[test]
fn label_width_thousand_cores() {
    assert_eq!(label_width(1000), 4);
}

#[test]
fn label_width_thousand_one_cores() {
    assert_eq!(label_width(1001), 5);
}

#[test]
fn util_chart_width_standard() {
    let cw = util_chart_width(40, 2);
    assert_eq!(cw, 40 - 2 - 9);
}

#[test]
fn util_chart_width_very_narrow() {
    let cw = util_chart_width(5, 2);
    assert_eq!(cw, 8);
}

#[test]
fn temp_chart_width_standard() {
    let cw = temp_chart_width(40, 4);
    assert_eq!(cw, 40 - 4 - 18);
}

#[test]
fn temp_chart_width_very_narrow() {
    let cw = temp_chart_width(10, 4);
    assert_eq!(cw, 8);
}

#[test]
fn mem_col_chart_width_standard() {
    let cw = mem_col_chart_width(40, 9);
    assert_eq!(cw, 40 - 12 - 9);
}

#[test]
fn mem_col_chart_width_very_narrow() {
    let cw = mem_col_chart_width(10, 9);
    assert_eq!(cw, 8);
}

#[test]
fn mem_col_chart_width_short_abs() {
    let cw = mem_col_chart_width(40, 12);
    assert_eq!(cw, 40 - 12 - 12);
}

#[test]
fn mem_col_chart_width_long_abs() {
    let cw = mem_col_chart_width(40, 15);
    assert_eq!(cw, 40 - 12 - 15);
}

// --- mem_abs_width / gpu_abs_width fixed width ---

#[test]
fn mem_abs_width_covers_all_used_values() {
    let mut state = ttop::memory::MemState::new();
    state.update(60);
    let fixed_w = mem_abs_width(&state);

    let test_values: &[u64] = &[
        0,
        1023,
        1024,
        1_048_575,
        1_048_576,
        state.current.mem_total_kb,
    ];
    for &used in test_values {
        if used > state.current.mem_total_kb {
            continue;
        }
        let text = format_mem_pair(used, state.current.mem_total_kb);
        assert!(
            text.len() <= fixed_w,
            "RAM format_mem_pair({used}, {}) = \"{text}\" ({} chars) exceeds fixed width {fixed_w}",
            state.current.mem_total_kb,
            text.len()
        );
    }
}

#[test]
fn mem_abs_width_equals_max_mem_pair_width() {
    let state = ttop::memory::MemState::new();
    let expected = max_mem_pair_width(state.current.mem_total_kb)
        .max(max_mem_pair_width(state.current.swap_total_kb));
    assert_eq!(mem_abs_width(&state), expected);
}

#[test]
fn gpu_abs_width_covers_all_used_values() {
    let gpu = ttop::gpu::GpuState::new();
    let fixed_w = gpu_abs_width(&gpu);

    let test_values: &[u64] = &[0, 1023, 1024, 1_048_575, gpu.current_mem_total_kb];
    for &used in test_values {
        if used > gpu.current_mem_total_kb {
            continue;
        }
        let text = format_mem_pair(used, gpu.current_mem_total_kb);
        assert!(
            text.len() <= fixed_w,
            "GPU format_mem_pair({used}, {}) = \"{text}\" ({} chars) exceeds fixed width {fixed_w}",
            gpu.current_mem_total_kb,
            text.len()
        );
    }
}

// --- core_columns ---

#[test]
fn core_columns_zero() {
    assert_eq!(core_columns(0), vec![0, 0]);
}

#[test]
fn core_columns_one() {
    assert_eq!(core_columns(1), vec![1, 0]);
}

#[test]
fn core_columns_two() {
    assert_eq!(core_columns(2), vec![1, 1]);
}

#[test]
fn core_columns_three() {
    assert_eq!(core_columns(3), vec![1, 1, 1]);
}

#[test]
fn core_columns_four_falls_back_to_two() {
    assert_eq!(core_columns(4), vec![2, 2]);
}

#[test]
fn core_columns_five() {
    assert_eq!(core_columns(5), vec![2, 2, 1]);
}

#[test]
fn core_columns_six() {
    assert_eq!(core_columns(6), vec![2, 2, 2]);
}

#[test]
fn core_columns_seven() {
    assert_eq!(core_columns(7), vec![3, 3, 1]);
}

#[test]
fn core_columns_eight() {
    assert_eq!(core_columns(8), vec![3, 3, 2]);
}

#[test]
fn core_columns_twelve() {
    assert_eq!(core_columns(12), vec![4, 4, 4]);
}

#[test]
fn core_columns_sixteen() {
    assert_eq!(core_columns(16), vec![6, 6, 4]);
}

#[test]
fn core_columns_twenty_four() {
    assert_eq!(core_columns(24), vec![8, 8, 8]);
}

#[test]
fn core_columns_sum_equals_input() {
    for n in 0..=128 {
        let cols = core_columns(n);
        assert_eq!(
            cols.iter().sum::<usize>(),
            n,
            "core_columns({n}) = {cols:?} does not sum to {n}"
        );
    }
}

#[test]
fn core_columns_no_empty_third_when_three_cols() {
    for n in 0..=128 {
        let cols = core_columns(n);
        if cols.len() == 3 {
            assert!(
                cols[2] > 0,
                "core_columns({n}) = {cols:?} has empty third column"
            );
        }
    }
}
