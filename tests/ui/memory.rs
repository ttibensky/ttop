use std::collections::VecDeque;

use ttop::ui::memory::{render_mem_col_first, render_mem_col_inner, render_mem_subtitle_line};

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
fn render_mem_col_first_contains_label_and_percentage() {
    let mut history = VecDeque::new();
    history.push_back(35.0);
    let mut buf = String::new();
    render_mem_col_first(&mut buf, "RAM", &history, 10, "5.1GB/16.0GB", 40, false);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("RAM"), "should contain RAM label");
    assert!(stripped.contains("35%"), "should contain percentage");
    assert!(stripped.contains("5.1GB/16.0GB"), "should contain absolute values");
}

#[test]
fn render_mem_col_first_starts_with_border() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_mem_col_first(&mut buf, "RAM", &history, 10, "0KB/0KB", 30, false);
    let stripped = strip_ansi(&buf);
    assert!(stripped.starts_with('│'), "first column should start with border");
}

#[test]
fn render_mem_col_first_dim_mode() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_mem_col_first(&mut buf, "SWP", &history, 10, "0.0GB/0.0GB", 40, true);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("SWP"), "should contain SWP label");
    assert!(stripped.contains("0%"), "dim mode should show 0%");
}

#[test]
fn render_mem_col_inner_contains_label_and_percentage() {
    let mut history = VecDeque::new();
    history.push_back(6.0);
    let mut buf = String::new();
    render_mem_col_inner(&mut buf, "SWP", &history, 10, "0.5GB/8.0GB", 35, false);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("SWP"), "should contain SWP label");
    assert!(stripped.contains("6%"), "should contain percentage");
    assert!(stripped.contains("0.5GB/8.0GB"), "should contain absolute values");
}

#[test]
fn render_mem_col_inner_does_not_start_with_border() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_mem_col_inner(&mut buf, "SWP", &history, 10, "0KB/0KB", 30, false);
    let stripped = strip_ansi(&buf);
    assert!(!stripped.starts_with('│'), "inner column should not start with border");
}

#[test]
fn render_mem_col_inner_dim_mode() {
    let history = VecDeque::new();
    let mut buf = String::new();
    render_mem_col_inner(&mut buf, "SWP", &history, 10, "0.0GB/0.0GB", 35, true);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("SWP"), "should contain SWP label");
    assert!(stripped.contains("0%"), "dim mode should show 0%");
}

#[test]
fn render_mem_subtitle_line_contains_all_titles() {
    let mut buf = String::new();
    render_mem_subtitle_line(&mut buf, 30, 30, 30);
    let stripped = strip_ansi(&buf);
    assert!(stripped.contains("RAM Utilization"), "should contain RAM Utilization");
    assert!(stripped.contains("Swap Utilization"), "should contain Swap Utilization");
    assert!(stripped.contains("Temperature"), "should contain Temperature");
}

#[test]
fn render_mem_subtitle_line_has_borders() {
    let mut buf = String::new();
    render_mem_subtitle_line(&mut buf, 30, 30, 30);
    let stripped = strip_ansi(&buf);
    let border_count = stripped.chars().filter(|&c| c == '│').count();
    assert_eq!(border_count, 4, "subtitle line should have 4 borders (left + 3 separators)");
}

#[test]
fn render_mem_subtitle_line_ends_with_newline() {
    let mut buf = String::new();
    render_mem_subtitle_line(&mut buf, 30, 30, 30);
    assert!(buf.ends_with("\r\n"), "subtitle line should end with CRLF");
}
