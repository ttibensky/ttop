use std::collections::VecDeque;
use std::fmt::Write;

use crate::cpu::temperature::{self, TempState};
use crate::cpu::utilization::CpuState;

pub const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_ORANGE: &str = "\x1b[38;5;208m";
pub const COLOR_RED: &str = "\x1b[31m";
const COLOR_DIM_GRAY: &str = "\x1b[90m";
const COLOR_DIM_CHART: &str = "\x1b[38;5;240m";
const COLOR_BOLD_CYAN: &str = "\x1b[1;36m";
const COLOR_WHITE: &str = "\x1b[37m";
const COLOR_RESET: &str = "\x1b[0m";

pub fn sparkline_char(pct: f64) -> char {
    let index = ((pct / 100.0) * 7.0).round().max(0.0) as usize;
    SPARKLINE_CHARS[index.min(7)]
}

pub fn sparkline_char_temp(temp_c: f64) -> char {
    let clamped = temp_c.clamp(30.0, 100.0);
    let index = ((clamped - 30.0) / 70.0 * 7.0).round() as usize;
    SPARKLINE_CHARS[index.min(7)]
}

pub fn utilization_color(pct: f64) -> &'static str {
    match pct as u32 {
        0..=25 => COLOR_GREEN,
        26..=50 => COLOR_YELLOW,
        51..=75 => COLOR_ORANGE,
        _ => COLOR_RED,
    }
}

pub fn temperature_color(temp_c: f64) -> &'static str {
    match temp_c as u32 {
        0..=49 => COLOR_GREEN,
        50..=69 => COLOR_YELLOW,
        70..=84 => COLOR_ORANGE,
        _ => COLOR_RED,
    }
}

pub fn label_width(core_count: usize) -> usize {
    if core_count == 0 {
        return 2;
    }
    let max_id = core_count - 1;
    let digits = if max_id == 0 {
        1
    } else {
        (max_id as f64).log10().floor() as usize + 1
    };
    1 + digits
}

pub fn temp_label_width(temp: &TempState) -> usize {
    if !temp.available() {
        return 3; // "N/A"
    }
    temp.labels().iter().map(|l| l.len()).max().unwrap_or(4).max(3)
}

/// Compute the chart width for the left (utilization) half.
/// Layout: "│ " + label + " " + chart + " " + "NNN%" + " │"
///          2   + lw    + 1   + cw    + 1   + 4      + 2  = lw + cw + 10
pub fn left_chart_width(half_cols: usize, lw: usize) -> usize {
    let fixed = lw + 10;
    if half_cols > fixed {
        half_cols - fixed
    } else {
        8
    }
}

/// Compute the chart width for the right (temperature) half.
/// Layout: " " + label + " " + chart + " NNN°C (NNN°F)" + " │"
///          1  + tlw   + 1   + cw    + 14                + 2  = tlw + cw + 18
pub fn right_chart_width(half_cols: usize, tlw: usize) -> usize {
    let fixed = tlw + 18;
    if half_cols > fixed {
        half_cols - fixed
    } else {
        8
    }
}

fn render_util_row(
    buf: &mut String,
    label: &str,
    lw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    half_width: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let current_color = utilization_color(current_pct);

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
    let _ = write!(buf, "{COLOR_WHITE}{:<width$}{COLOR_RESET} ", label, width = lw);

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

    // pad to fill left half
    let used = 2 + lw + 1 + cw + 1 + 4;
    let pad = half_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

fn render_temp_row(
    buf: &mut String,
    label: &str,
    tlw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    half_width: usize,
) {
    let current_c = history.back().copied();

    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}{:>width$}{COLOR_RESET} ", label, width = tlw);

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }
    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char_temp(val);
        let color = temperature_color(val);
        let _ = write!(buf, "{color}{ch}{COLOR_RESET}");
    }

    if let Some(c) = current_c {
        let f = temperature::celsius_to_fahrenheit(c);
        let color = temperature_color(c);
        let _ = write!(buf, " {color}{:>3.0}°C ({:>3.0}°F){COLOR_RESET}", c, f);
    } else {
        let _ = write!(buf, " {COLOR_DIM_GRAY}N/A°C (N/A°F){COLOR_RESET}");
    }

    // pad + right border
    // used: 1 + tlw + 1 + cw + 14 = tlw + cw + 16, plus 2 for " │"
    let used = 1 + tlw + 1 + cw + 14;
    let pad = half_width.saturating_sub(used + 2);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");
}

fn render_empty_right_half(buf: &mut String, half_width: usize) {
    let inner = half_width.saturating_sub(1);
    for _ in 0..inner {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
}

fn render_na_temp_row(buf: &mut String, tlw: usize, cw: usize, half_width: usize) {
    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}{:>width$}{COLOR_RESET} ", "N/A", width = tlw);

    for _ in 0..cw {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }

    let _ = write!(buf, " {COLOR_DIM_GRAY}N/A°C (N/A°F){COLOR_RESET}");

    let used = 1 + tlw + 1 + cw + 14;
    let pad = half_width.saturating_sub(used + 2);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");
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
        let title_chars = t.len() + 2;
        let fill = (width as usize).saturating_sub(3 + title_chars);
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

fn render_subtitle_line(
    buf: &mut String,
    left_title: &str,
    right_title: &str,
    left_half: usize,
    terminal_cols: u16,
) {
    let left_inner = left_half.saturating_sub(1); // exclude left border
    let right_inner = (terminal_cols as usize).saturating_sub(left_half + 2); // exclude center + right borders

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let left_pad = left_inner.saturating_sub(left_title.len()) / 2;
    let left_remaining = left_inner.saturating_sub(left_pad + left_title.len());
    for _ in 0..left_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{left_title}{COLOR_RESET}");
    for _ in 0..left_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let right_pad = right_inner.saturating_sub(right_title.len()) / 2;
    let right_remaining = right_inner.saturating_sub(right_pad + right_title.len());
    for _ in 0..right_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{right_title}{COLOR_RESET}");
    for _ in 0..right_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

fn render_separator_line(buf: &mut String, left_half: usize, terminal_cols: u16) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for _ in 0..left_half.saturating_sub(1) {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    let right_inner = (terminal_cols as usize).saturating_sub(left_half + 2);
    for _ in 0..right_inner {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

pub fn render_frame(cpu: &CpuState, temp: &TempState, cols: u16, rows: u16) -> String {
    let total_inner = (cols as usize).saturating_sub(2);
    let left_half = total_inner / 2 + 1; // +1 for the left border │
    let right_half = (cols as usize).saturating_sub(left_half + 1); // -1 for center separator │

    let lw = label_width(cpu.core_count());
    let lcw = left_chart_width(left_half, lw);

    let tlw = temp_label_width(temp);
    let rcw = right_chart_width(right_half, tlw);

    let row_count = cpu.core_count();
    let temp_rows = if temp.available() {
        temp.sensor_count()
    } else {
        1 // N/A row
    };

    let mut buf = String::with_capacity((cols as usize) * (rows as usize));
    buf.push_str("\x1b[H");

    // CPU section top border
    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("CPU"));
    render_subtitle_line(&mut buf, "Utilization", "Temperature", left_half, cols);

    for i in 0..row_count {
        let label = format!("#{}", i);
        render_util_row(&mut buf, &label, lw, &cpu.histories[i], lcw, left_half);

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        if i < temp_rows {
            if temp.available() {
                let labels = temp.labels();
                render_temp_row(
                    &mut buf,
                    labels[i],
                    tlw,
                    &temp.histories[i],
                    rcw,
                    right_half,
                );
            } else {
                render_na_temp_row(&mut buf, tlw, rcw, right_half);
            }
        } else {
            render_empty_right_half(&mut buf, right_half);
        }

        buf.push_str("\r\n");
    }

    render_separator_line(&mut buf, left_half, cols);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // fill remaining rows
    let cpu_lines = row_count + 4;
    let remaining_lines = (rows as usize).saturating_sub(cpu_lines + 1);
    for _ in 0..remaining_lines {
        let _ = write!(buf, "\x1b[K\r\n");
    }

    // status bar
    let status = "q: quit  ttop v0.1";
    let padding = (cols as usize).saturating_sub(status.len());
    for _ in 0..padding {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}{status}{COLOR_RESET}");

    buf
}
