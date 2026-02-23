use std::collections::VecDeque;
use std::fmt::Write;

use crate::disk::io::format_rate;

use super::colors::*;

pub fn render_disk_space_col_first(
    buf: &mut String,
    label: &str,
    lw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    section_width: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let color = utilization_color(current_pct);

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
    let _ = write!(buf, "{COLOR_WHITE}{:<width$}{COLOR_RESET} ", label, width = lw);

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

    let used = 2 + lw + 1 + cw + 1 + abs_w + 2 + 4;
    let pad = section_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

#[allow(clippy::too_many_arguments)]
pub fn render_disk_io_col_right(
    buf: &mut String,
    label: &str,
    lw: usize,
    history: &VecDeque<f64>,
    cw: usize,
    max_observed: f64,
    rw: usize,
    half_width: usize,
) {
    let current_val = history.back().copied().unwrap_or(0.0);
    let color = io_color(current_val, max_observed);

    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}{:<width$}{COLOR_RESET} ", label, width = lw);

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    for _ in 0..empty_slots {
        let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
    }
    for &val in history.iter().skip(data_len.saturating_sub(cw)) {
        let ch = sparkline_char_scaled(val, max_observed);
        let c = io_color(val, max_observed);
        let _ = write!(buf, "{c}{ch}{COLOR_RESET}");
    }

    let rate_text = format_rate(current_val);
    let _ = write!(buf, " {color}{:>width$}{COLOR_RESET}", rate_text, width = rw);

    let used = 1 + lw + 1 + cw + 1 + rw;
    let pad = half_width.saturating_sub(used + 2);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}");
}

pub fn render_disk_subtitle_line(
    buf: &mut String,
    space_span: usize,
    io_col: usize,
) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let space_title = "Space";
    let space_pad = space_span.saturating_sub(space_title.len()) / 2;
    let space_remaining = space_span.saturating_sub(space_pad + space_title.len());
    for _ in 0..space_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{space_title}{COLOR_RESET}");
    for _ in 0..space_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let io_title = "I/O";
    let io_pad = io_col.saturating_sub(io_title.len()) / 2;
    let io_remaining = io_col.saturating_sub(io_pad + io_title.len());
    for _ in 0..io_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{io_title}{COLOR_RESET}");
    for _ in 0..io_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}
