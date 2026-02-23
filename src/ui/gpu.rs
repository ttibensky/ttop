use std::collections::VecDeque;
use std::fmt::Write;

use crate::cpu::temperature;

use super::colors::*;

pub fn render_gpu_util_col_first(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    section_width: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let color = utilization_color(current_pct);

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
    let _ = write!(buf, "{COLOR_WHITE}USE{COLOR_RESET} ");

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }
    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char(val);
        let c = utilization_color(val);
        let _ = write!(buf, "{c}{ch}{COLOR_RESET}");
    }

    let _ = write!(buf, " {color}{:>3.0}%{COLOR_RESET}", current_pct);

    let used = 2 + 3 + 1 + cw + 1 + 4;
    let pad = section_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_gpu_mem_col_inner(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    col_width: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let color = utilization_color(current_pct);

    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}MEM{COLOR_RESET} ");

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }
    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char(val);
        let c = utilization_color(val);
        let _ = write!(buf, "{c}{ch}{COLOR_RESET}");
    }

    let abs_w = abs_formatted.len();
    let _ = write!(buf, " {color}{abs_formatted}{COLOR_RESET}");
    let _ = write!(buf, "  {color}{:>3.0}%{COLOR_RESET}", current_pct);

    let used = 1 + 3 + 1 + cw + 1 + abs_w + 2 + 4;
    let pad = col_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_gpu_temp_col_right(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    half_width: usize,
) {
    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}TMP{COLOR_RESET} ");

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }
    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char_temp(val);
        let c = temperature_color(val);
        let _ = write!(buf, "{c}{ch}{COLOR_RESET}");
    }

    if let Some(&c) = history.back() {
        let f = temperature::celsius_to_fahrenheit(c);
        let color = temperature_color(c);
        let _ = write!(buf, " {color}{:>3.0}°C ({:>3.0}°F){COLOR_RESET}", c, f);
    } else {
        let _ = write!(buf, " {COLOR_DIM_GRAY}N/A°C (N/A°F){COLOR_RESET}");
    }

    let used = 1 + 3 + 1 + cw + 14;
    let pad = half_width.saturating_sub(used + 2);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");
}

pub fn render_gpu_subtitle_line(
    buf: &mut String,
    col1: usize,
    col2: usize,
    col3: usize,
) {
    let titles = [
        ("GPU Utilization", col1),
        ("VRAM Utilization", col2),
        ("Temperature", col3),
    ];

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for (title, col_width) in titles {
        let pad_left = col_width.saturating_sub(title.len()) / 2;
        let pad_right = col_width.saturating_sub(pad_left + title.len());
        for _ in 0..pad_left {
            buf.push(' ');
        }
        let _ = write!(buf, "{COLOR_BOLD_CYAN}{title}{COLOR_RESET}");
        for _ in 0..pad_right {
            buf.push(' ');
        }
        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    }
    buf.push_str("\r\n");
}
