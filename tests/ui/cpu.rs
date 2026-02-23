use std::collections::VecDeque;

use ttop::ui::cpu::{
    render_empty_col, render_empty_first_col, render_empty_right_half, render_na_temp_row,
    render_temp_row, render_util_row, render_util_row_inner,
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
fn render_util_row_contains_label_and_percentage() {
    let mut history = VecDeque::new();
    history.push_back(42.0);
    let mut buf = String::new();
    render_util_row(&mut buf, "#0", 2, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("#0"), "should contain core label");
    assert!(stripped.contains("42%"), "should contain percentage");
}

#[test]
fn render_util_row_empty_history_shows_zero() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_util_row(&mut buf, "#0", 2, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("0%"), "empty history should show 0%");
}

#[test]
fn render_util_row_inner_contains_label_and_percentage() {
    let mut history = VecDeque::new();
    history.push_back(75.0);
    let mut buf = String::new();
    render_util_row_inner(&mut buf, "#3", 2, &history, 10, 25);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("#3"), "should contain core label");
    assert!(stripped.contains("75%"), "should contain percentage");
}

#[test]
fn render_util_row_inner_does_not_start_with_border() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_util_row_inner(&mut buf, "#1", 2, &history, 10, 25);
    let stripped = strip_ansi(&buf);
    assert!(!stripped.starts_with('│'), "inner row should not start with border");
}

#[test]
fn render_empty_first_col_has_border() {
    let mut buf = String::new();
    render_empty_first_col(&mut buf, 20);
    let stripped = strip_ansi(&buf);
    assert!(stripped.starts_with('│'), "should start with left border");
    assert_eq!(stripped.chars().count(), 20, "should be exactly section_width chars");
}

#[test]
fn render_empty_col_is_all_spaces() {
    let mut buf = String::new();
    render_empty_col(&mut buf, 15);
    assert_eq!(buf.len(), 15);
    assert!(buf.chars().all(|c| c == ' '), "should be all spaces");
}

#[test]
fn render_empty_right_half_ends_with_border() {
    let mut buf = String::new();
    render_empty_right_half(&mut buf, 20);
    let stripped = strip_ansi(&buf);
    assert!(stripped.ends_with('│'), "should end with right border");
}

#[test]
fn render_temp_row_contains_celsius_and_fahrenheit() {
    let mut history = VecDeque::new();
    history.push_back(65.0);
    let mut buf = String::new();
    render_temp_row(&mut buf, "Tctl", 4, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("°C"), "should contain °C");
    assert!(stripped.contains("°F"), "should contain °F");
    assert!(stripped.contains("Tctl"), "should contain sensor label");
}

#[test]
fn render_temp_row_empty_history_shows_na() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_temp_row(&mut buf, "Tctl", 4, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("N/A"), "empty history should show N/A");
}

#[test]
fn render_temp_row_ends_with_border() {
    let mut history = VecDeque::new();
    history.push_back(50.0);
    let mut buf = String::new();
    render_temp_row(&mut buf, "Tctl", 4, &history, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.ends_with('│'), "should end with right border");
}

#[test]
fn render_na_temp_row_shows_na() {
    let mut buf = String::new();
    render_na_temp_row(&mut buf, 3, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("N/A°C"), "should contain N/A°C");
    assert!(stripped.contains("N/A°F"), "should contain N/A°F");
}

#[test]
fn render_na_temp_row_ends_with_border() {
    let mut buf = String::new();
    render_na_temp_row(&mut buf, 3, 10, 40);
    let stripped = strip_ansi(&buf);
    assert!(stripped.ends_with('│'), "should end with right border");
}

#[test]
fn render_util_row_sparkline_fills_chart_width() {
    let mut history = VecDeque::new();
    for i in 0..5 {
        history.push_back(i as f64 * 20.0);
    }
    let mut buf = String::new();
    render_util_row(&mut buf, "#0", 2, &history, 10, 30);
    let stripped = strip_ansi(&buf);
    let sparkline_chars: Vec<char> = "▁▂▃▄▅▆▇█".chars().collect();
    let sparkline_count = stripped.chars().filter(|c| sparkline_chars.contains(c)).count();
    assert_eq!(sparkline_count, 10, "chart should fill the full chart width with sparkline chars");
}
