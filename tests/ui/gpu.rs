use std::collections::VecDeque;

use ttop::ui::gpu::{
    render_gpu_mem_col_inner, render_gpu_subtitle_line, render_gpu_temp_col_right,
    render_gpu_util_col_first,
};

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

#[test]
fn render_gpu_util_col_first_contains_use_label() {
    let mut history = VecDeque::new();
    history.push_back(72.0);
    let mut buf = String::new();
    render_gpu_util_col_first(&mut buf, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("USE"), "should contain USE label");
    assert!(stripped.contains("72%"), "should contain percentage");
}

#[test]
fn render_gpu_util_col_first_starts_with_border() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_gpu_util_col_first(&mut buf, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.starts_with('│'), "first column should start with border");
}

#[test]
fn render_gpu_util_col_first_empty_history_shows_zero() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_gpu_util_col_first(&mut buf, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("0%"), "empty history should show 0%");
}

#[test]
fn render_gpu_mem_col_inner_contains_mem_label_and_abs() {
    let mut history = VecDeque::new();
    history.push_back(40.0);
    let mut buf = String::new();
    render_gpu_mem_col_inner(&mut buf, &history, 10, "4.2GB/24.0GB", 35);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("MEM"), "should contain MEM label");
    assert!(stripped.contains("40%"), "should contain percentage");
    assert!(stripped.contains("4.2GB/24.0GB"), "should contain absolute values");
}

#[test]
fn render_gpu_mem_col_inner_does_not_start_with_border() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_gpu_mem_col_inner(&mut buf, &history, 10, "0KB/0KB", 30);
    let stripped = strip_ansi(&buf);
    assert!(!stripped.starts_with('│'), "inner column should not start with border");
}

#[test]
fn render_gpu_temp_col_right_contains_celsius_and_fahrenheit() {
    let mut history = VecDeque::new();
    history.push_back(52.0);
    let mut buf = String::new();
    render_gpu_temp_col_right(&mut buf, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("TMP"), "should contain TMP label");
    assert!(stripped.contains("°C"), "should contain °C");
    assert!(stripped.contains("°F"), "should contain °F");
}

#[test]
fn render_gpu_temp_col_right_empty_history_shows_na() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_gpu_temp_col_right(&mut buf, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("N/A"), "empty history should show N/A");
}

#[test]
fn render_gpu_temp_col_right_ends_with_border() {
    let mut history = VecDeque::new();
    history.push_back(50.0);
    let mut buf = String::new();
    render_gpu_temp_col_right(&mut buf, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.ends_with('│'), "should end with right border");
}

#[test]
fn render_gpu_subtitle_line_contains_all_titles() {
    let mut buf = String::new();
    render_gpu_subtitle_line(&mut buf, 30, 30, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("GPU Utilization"), "should contain GPU Utilization");
    assert!(stripped.contains("VRAM Utilization"), "should contain VRAM Utilization");
    assert!(stripped.contains("Temperature"), "should contain Temperature");
}

#[test]
fn render_gpu_subtitle_line_has_borders() {
    let mut buf = String::new();
    render_gpu_subtitle_line(&mut buf, 30, 30, 30);
    let stripped = strip_ansi(&buf);
    let border_count = stripped.chars().filter(|&c| c == '│').count();
    assert_eq!(border_count, 4, "subtitle line should have 4 borders");
}

#[test]
fn render_gpu_subtitle_line_ends_with_newline() {
    let mut buf = String::new();
    render_gpu_subtitle_line(&mut buf, 30, 30, 30);
    assert!(buf.ends_with("\r\n"), "subtitle line should end with CRLF");
}
