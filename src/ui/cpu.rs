use std::collections::VecDeque;
use std::fmt::Write;

use crate::cpu::temperature;

use super::colors::*;

pub fn render_util_row(
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

    let used = 2 + lw + 1 + cw + 1 + 4;
    let pad = half_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_util_row_inner(
    buf: &mut String,
    label: &str,
    lw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    col_width: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let current_color = utilization_color(current_pct);

    let _ = write!(buf, " {COLOR_WHITE}{:<width$}{COLOR_RESET} ", label, width = lw);

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

    let used = 1 + lw + 1 + cw + 1 + 4;
    let pad = col_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_empty_first_col(buf: &mut String, section_width: usize) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    let inner = section_width.saturating_sub(1);
    for _ in 0..inner {
        buf.push(' ');
    }
}

pub fn render_empty_col(buf: &mut String, col_width: usize) {
    for _ in 0..col_width {
        buf.push(' ');
    }
}

pub fn render_temp_row(
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

    let used = 1 + tlw + 1 + cw + 14;
    let pad = half_width.saturating_sub(used + 2);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");
}

pub fn render_empty_right_half(buf: &mut String, half_width: usize) {
    let inner = half_width.saturating_sub(1);
    for _ in 0..inner {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
}

pub fn render_na_temp_row(buf: &mut String, tlw: usize, cw: usize, half_width: usize) {
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
