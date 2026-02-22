use std::collections::VecDeque;
use std::fmt::Write;

use crate::cpu::CpuState;

const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

const COLOR_GREEN: &str = "\x1b[32m";
const COLOR_YELLOW: &str = "\x1b[33m";
const COLOR_ORANGE: &str = "\x1b[38;5;208m";
const COLOR_RED: &str = "\x1b[31m";
const COLOR_DIM_GRAY: &str = "\x1b[90m";
const COLOR_DIM_CHART: &str = "\x1b[38;5;240m";
const COLOR_BOLD_CYAN: &str = "\x1b[1;36m";
const COLOR_WHITE: &str = "\x1b[37m";
const COLOR_RESET: &str = "\x1b[0m";

fn sparkline_char(pct: f64) -> char {
    let index = ((pct / 100.0) * 7.0).round() as usize;
    SPARKLINE_CHARS[index.min(7)]
}

fn utilization_color(pct: f64) -> &'static str {
    match pct as u32 {
        0..=25 => COLOR_GREEN,
        26..=50 => COLOR_YELLOW,
        51..=75 => COLOR_ORANGE,
        _ => COLOR_RED,
    }
}

pub fn chart_width(terminal_cols: u16, label_width: usize) -> usize {
    // Row layout: "│ " + label + " " + chart + " " + percentage(4) + " │"
    //             2    + label  + 1   + chart + 1   + 4             + 2  = label + chart + 10
    let fixed = label_width + 10;
    if (terminal_cols as usize) > fixed {
        terminal_cols as usize - fixed
    } else {
        10 // minimum chart width
    }
}

pub fn label_width(core_count: usize) -> usize {
    if core_count == 0 {
        return 2;
    }
    let max_id = core_count - 1;
    let digits = if max_id == 0 { 1 } else { (max_id as f64).log10().floor() as usize + 1 };
    digits.max(2)
}

fn render_sparkline_row(
    buf: &mut String,
    label: &str,
    lw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    terminal_cols: u16,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let current_color = utilization_color(current_pct);

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
    let _ = write!(buf, "{COLOR_WHITE}{:>width$}{COLOR_RESET} ", label, width = lw);

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }

    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char(val);
        let color = utilization_color(val);
        let _ = write!(buf, "{color}{ch}{COLOR_RESET}");
    }

    let _ = write!(buf, " {current_color}{:>3.0}%{COLOR_RESET}", current_pct);
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");

    // clear any leftover characters on the line
    let used = 2 + lw + 1 + cw + 1 + 4 + 2;
    let remaining = (terminal_cols as usize).saturating_sub(used);
    for _ in 0..remaining {
        buf.push(' ');
    }

    buf.push_str("\r\n");
}

fn render_horizontal_border(
    buf: &mut String,
    left: char,
    right: char,
    width: u16,
    title: Option<&str>,
) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}{left}─");
    if let Some(t) = title {
        let _ = write!(buf, " {COLOR_BOLD_CYAN}{t}{COLOR_RESET}{COLOR_DIM_GRAY} ");
        let title_chars = t.len() + 2; // spaces around title
        let fill = (width as usize).saturating_sub(4 + title_chars);
        for _ in 0..fill {
            buf.push('─');
        }
    } else {
        let fill = (width as usize).saturating_sub(3);
        for _ in 0..fill {
            buf.push('─');
        }
    }
    let _ = write!(buf, "{right}{COLOR_RESET}\r\n");
}

fn render_empty_line(buf: &mut String, terminal_cols: u16) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    let inner = (terminal_cols as usize).saturating_sub(2);
    for _ in 0..inner {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

pub fn render_frame(cpu: &CpuState, cols: u16, rows: u16) -> String {
    let lw = label_width(cpu.core_count());
    let cw = chart_width(cols, lw);

    let mut buf = String::with_capacity((cols as usize) * (rows as usize));

    // move cursor to top-left
    buf.push_str("\x1b[H");

    // CPU section
    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("CPU"));
    render_empty_line(&mut buf, cols);

    for i in 0..cpu.core_count() {
        let label = format!("{}", i);
        render_sparkline_row(&mut buf, &label, lw, &cpu.histories[i], cw, cols);
    }

    render_empty_line(&mut buf, cols);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // fill remaining rows with blank lines, then status bar on the last line
    let cpu_lines = cpu.core_count() + 4; // top border + empty + cores + empty + bottom border
    let remaining_lines = (rows as usize).saturating_sub(cpu_lines + 1);
    for _ in 0..remaining_lines {
        let _ = write!(buf, "\x1b[K\r\n");
    }

    // status bar on the last line
    let status = "q: quit  ttop v0.1";
    let padding = (cols as usize).saturating_sub(status.len());
    for _ in 0..padding {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}{status}{COLOR_RESET}");

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(label_width(11), 2);
    }

    #[test]
    fn label_width_hundred_cores() {
        assert_eq!(label_width(100), 2);
    }

    #[test]
    fn label_width_hundred_one_cores() {
        assert_eq!(label_width(101), 3);
    }

    #[test]
    fn label_width_thousand_cores() {
        assert_eq!(label_width(1000), 3);
    }

    #[test]
    fn label_width_thousand_one_cores() {
        assert_eq!(label_width(1001), 4);
    }

    #[test]
    fn chart_width_standard_terminal() {
        let lw = 2;
        let cw = chart_width(80, lw);
        assert_eq!(cw, 80 - lw - 10);
    }

    #[test]
    fn chart_width_wide_terminal() {
        let lw = 2;
        let cw = chart_width(200, lw);
        assert_eq!(cw, 200 - lw - 10);
    }

    #[test]
    fn chart_width_very_narrow_terminal_returns_minimum() {
        let cw = chart_width(5, 2);
        assert_eq!(cw, 10);
    }

    #[test]
    fn chart_width_exact_minimum() {
        let lw = 2;
        let cw = chart_width((lw + 10) as u16, lw);
        assert_eq!(cw, 10);
    }

    #[test]
    fn render_sparkline_row_contains_label() {
        let mut buf = String::new();
        let mut history = VecDeque::new();
        history.push_back(42.0);
        render_sparkline_row(&mut buf, "0", 2, &history, 20, 40);
        let stripped = strip_ansi(&buf);
        assert!(stripped.contains("0"), "row should contain the label");
    }

    #[test]
    fn render_sparkline_row_contains_percentage() {
        let mut buf = String::new();
        let mut history = VecDeque::new();
        history.push_back(73.0);
        render_sparkline_row(&mut buf, "5", 2, &history, 20, 40);
        let stripped = strip_ansi(&buf);
        assert!(stripped.contains("73%"), "row should contain '73%', got: {}", stripped);
    }

    #[test]
    fn render_sparkline_row_empty_history_shows_zero() {
        let mut buf = String::new();
        let history = VecDeque::new();
        render_sparkline_row(&mut buf, "0", 2, &history, 10, 30);
        let stripped = strip_ansi(&buf);
        assert!(stripped.contains("0%"), "empty history should show 0%");
    }

    #[test]
    fn render_horizontal_border_with_title_contains_title() {
        let mut buf = String::new();
        render_horizontal_border(&mut buf, '╭', '╮', 80, Some("CPU"));
        let stripped = strip_ansi(&buf);
        assert!(stripped.contains("CPU"));
        assert!(stripped.starts_with('╭'));
        assert!(stripped.trim_end().ends_with('╮'));
    }

    #[test]
    fn render_horizontal_border_without_title() {
        let mut buf = String::new();
        render_horizontal_border(&mut buf, '╰', '╯', 80, None);
        let stripped = strip_ansi(&buf);
        assert!(stripped.starts_with('╰'));
        assert!(stripped.trim_end().ends_with('╯'));
        assert!(!stripped.contains("CPU"));
    }

    #[test]
    fn render_empty_line_has_borders() {
        let mut buf = String::new();
        render_empty_line(&mut buf, 80);
        let stripped = strip_ansi(&buf);
        assert!(stripped.starts_with('│'));
        assert!(stripped.trim_end().ends_with('│'));
    }

    #[test]
    fn render_frame_contains_cpu_section() {
        let state = CpuState::new();
        let frame = render_frame(&state, 80, 40);
        let stripped = strip_ansi(&frame);
        assert!(stripped.contains("CPU"), "frame should contain CPU section header");
    }

    #[test]
    fn render_frame_contains_status_bar() {
        let state = CpuState::new();
        let frame = render_frame(&state, 80, 40);
        let stripped = strip_ansi(&frame);
        assert!(stripped.contains("q: quit"), "frame should contain quit hint");
        assert!(stripped.contains("ttop v0.1"), "frame should contain version");
    }

    #[test]
    fn render_frame_contains_all_core_labels() {
        let state = CpuState::new();
        let frame = render_frame(&state, 80, 60);
        let stripped = strip_ansi(&frame);
        for i in 0..state.core_count() {
            assert!(
                stripped.contains(&format!("{}",  i)),
                "frame should contain label for core {}",
                i
            );
        }
    }

    #[test]
    fn render_frame_has_box_drawing_chars() {
        let state = CpuState::new();
        let frame = render_frame(&state, 80, 40);
        let stripped = strip_ansi(&frame);
        assert!(stripped.contains('╭'));
        assert!(stripped.contains('╮'));
        assert!(stripped.contains('╰'));
        assert!(stripped.contains('╯'));
        assert!(stripped.contains('│'));
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
}
