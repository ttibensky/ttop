use ttop::cpu::temperature::TempState;
use ttop::cpu::utilization::CpuState;
use ttop::gpu::GpuState;
use ttop::memory::{MemState, MemTempState};
use ttop::ui::{
    core_columns, label_width, mem_col_chart_width, render_frame, sparkline_char,
    sparkline_char_temp, temp_chart_width, temperature_color, util_chart_width, utilization_color,
    COLOR_GREEN, COLOR_ORANGE, COLOR_RED, COLOR_YELLOW, SPARKLINE_CHARS,
};

#[test]
fn sparkline_char_at_zero_percent() {
    assert_eq!(sparkline_char(0.0), '▁');
}

#[test]
fn sparkline_char_at_hundred_percent() {
    assert_eq!(sparkline_char(100.0), '█');
}

#[test]
fn sparkline_char_at_fifty_percent() {
    let ch = sparkline_char(50.0);
    assert!(
        ['▃', '▄', '▅'].contains(&ch),
        "50% should map to a mid-range block, got '{}'",
        ch
    );
}

#[test]
fn sparkline_char_boundaries() {
    assert_eq!(sparkline_char(0.0), SPARKLINE_CHARS[0]);
    assert_eq!(sparkline_char(100.0), SPARKLINE_CHARS[7]);
}

#[test]
fn sparkline_char_negative_clamps_to_lowest() {
    assert_eq!(sparkline_char(-10.0), '▁');
}

#[test]
fn sparkline_char_over_hundred_clamps_to_highest() {
    assert_eq!(sparkline_char(150.0), '█');
}

#[test]
fn sparkline_char_temp_at_30() {
    assert_eq!(sparkline_char_temp(30.0), '▁');
}

#[test]
fn sparkline_char_temp_at_100() {
    assert_eq!(sparkline_char_temp(100.0), '█');
}

#[test]
fn sparkline_char_temp_at_65() {
    let ch = sparkline_char_temp(65.0);
    assert!(
        ['▃', '▄', '▅'].contains(&ch),
        "65°C should be mid-range, got '{}'",
        ch
    );
}

#[test]
fn sparkline_char_temp_below_30_clamps() {
    assert_eq!(sparkline_char_temp(10.0), '▁');
}

#[test]
fn sparkline_char_temp_above_100_clamps() {
    assert_eq!(sparkline_char_temp(120.0), '█');
}

#[test]
fn utilization_color_green_range() {
    assert_eq!(utilization_color(0.0), COLOR_GREEN);
    assert_eq!(utilization_color(12.0), COLOR_GREEN);
    assert_eq!(utilization_color(25.0), COLOR_GREEN);
}

#[test]
fn utilization_color_yellow_range() {
    assert_eq!(utilization_color(26.0), COLOR_YELLOW);
    assert_eq!(utilization_color(38.0), COLOR_YELLOW);
    assert_eq!(utilization_color(50.0), COLOR_YELLOW);
}

#[test]
fn utilization_color_orange_range() {
    assert_eq!(utilization_color(51.0), COLOR_ORANGE);
    assert_eq!(utilization_color(63.0), COLOR_ORANGE);
    assert_eq!(utilization_color(75.0), COLOR_ORANGE);
}

#[test]
fn utilization_color_red_range() {
    assert_eq!(utilization_color(76.0), COLOR_RED);
    assert_eq!(utilization_color(90.0), COLOR_RED);
    assert_eq!(utilization_color(100.0), COLOR_RED);
}

#[test]
fn temperature_color_green_range() {
    assert_eq!(temperature_color(30.0), COLOR_GREEN);
    assert_eq!(temperature_color(49.0), COLOR_GREEN);
}

#[test]
fn temperature_color_yellow_range() {
    assert_eq!(temperature_color(50.0), COLOR_YELLOW);
    assert_eq!(temperature_color(69.0), COLOR_YELLOW);
}

#[test]
fn temperature_color_orange_range() {
    assert_eq!(temperature_color(70.0), COLOR_ORANGE);
    assert_eq!(temperature_color(84.0), COLOR_ORANGE);
}

#[test]
fn temperature_color_red_range() {
    assert_eq!(temperature_color(85.0), COLOR_RED);
    assert_eq!(temperature_color(100.0), COLOR_RED);
}

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
fn render_frame_contains_cpu_section() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("CPU"), "frame should contain CPU header");
}

#[test]
fn render_frame_contains_subtitles() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(
        stripped.contains("Utilization"),
        "frame should contain 'Utilization' subtitle"
    );
    assert!(
        stripped.contains("Temperature"),
        "frame should contain 'Temperature' subtitle"
    );
}

#[test]
fn render_frame_contains_status_bar() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("q: quit"));
    assert!(stripped.contains("ttop v0.1"));
}

#[test]
fn render_frame_contains_all_core_labels() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    for i in 0..cpu.core_count() {
        assert!(
            stripped.contains(&format!("#{}", i)),
            "frame should contain label for core #{}",
            i
        );
    }
}

#[test]
fn render_frame_has_box_drawing_chars() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains('╭'));
    assert!(stripped.contains('╮'));
    assert!(stripped.contains('╰'));
    assert!(stripped.contains('╯'));
}

#[test]
fn render_frame_contains_vertical_separator() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    let lines: Vec<&str> = stripped.lines().collect();
    if lines.len() > 2 {
        let data_line = lines[2];
        let pipe_count = data_line.chars().filter(|&c| c == '│').count();
        assert!(
            pipe_count >= 3,
            "data row should have left border, separator, and right border"
        );
    }
}

#[test]
fn render_frame_shows_temp_or_na() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    let has_temp = stripped.contains("°C") && stripped.contains("°F");
    let has_na = stripped.contains("N/A");
    assert!(has_temp || has_na, "frame should show temperature or N/A");
}

#[test]
fn render_frame_contains_memory_section() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("Memory"), "frame should contain Memory header");
}

#[test]
fn render_frame_contains_ram_and_swp_labels() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("RAM"), "frame should contain RAM label");
    assert!(stripped.contains("SWP"), "frame should contain SWP label");
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

#[test]
fn render_frame_memory_has_three_column_subtitles() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(
        stripped.contains("RAM Utilization"),
        "frame should contain 'RAM Utilization' subtitle"
    );
    assert!(
        stripped.contains("Swap Utilization"),
        "frame should contain 'Swap Utilization' subtitle"
    );
}

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

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.next() {
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
