use std::collections::VecDeque;

use ttop::ui::disk::{render_disk_io_col_right, render_disk_space_col_first, render_disk_subtitle_line};

#[test]
fn render_disk_subtitle_line_contains_titles() {
    let mut buf = String::new();
    render_disk_subtitle_line(&mut buf, 40, 30);
    assert!(buf.contains("Space"), "should contain Space subtitle");
    assert!(buf.contains("I/O"), "should contain I/O subtitle");
}

#[test]
fn render_disk_space_col_first_smoke() {
    let mut buf = String::new();
    let mut history = VecDeque::new();
    history.push_back(45.0);
    render_disk_space_col_first(
        &mut buf,
        "/",
        5,
        &history,
        10,
        "120.5GB/500.0GB",
        50,
    );
    assert!(!buf.is_empty());
    assert!(buf.contains('/'), "should contain mount label");
    assert!(buf.contains("45%"), "should contain percentage");
}

#[test]
fn render_disk_space_col_first_empty_history() {
    let mut buf = String::new();
    let history = VecDeque::new();
    render_disk_space_col_first(
        &mut buf,
        "/home",
        5,
        &history,
        10,
        "0KB/500.0GB",
        50,
    );
    assert!(!buf.is_empty());
    assert!(buf.contains("0%"), "should show 0% for empty history");
}

#[test]
fn render_disk_io_col_right_smoke() {
    let mut buf = String::new();
    let mut history = VecDeque::new();
    history.push_back(1024.0 * 1024.0 * 50.0);
    render_disk_io_col_right(
        &mut buf,
        "sdaR",
        5,
        &history,
        10,
        1024.0 * 1024.0 * 100.0,
        9,
        40,
    );
    assert!(!buf.is_empty());
    assert!(buf.contains("sda"), "should contain device label");
    assert!(buf.contains("MB/s"), "should contain rate unit");
}

#[test]
fn render_disk_io_col_right_zero_throughput() {
    let mut buf = String::new();
    let mut history = VecDeque::new();
    history.push_back(0.0);
    render_disk_io_col_right(
        &mut buf,
        "sdaW",
        5,
        &history,
        10,
        0.0,
        6,
        40,
    );
    assert!(!buf.is_empty());
    assert!(buf.contains("0B/s"), "should show zero rate");
}
