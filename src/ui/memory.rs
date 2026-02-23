use std::fmt::Write;

use super::colors::*;

pub fn render_mem_col_first(
    buf: &mut String,
    label: &str,
    history: &std::collections::VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    section_width: usize,
    dim: bool,
) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
    let _ = write!(buf, "{COLOR_WHITE}{:<3}{COLOR_RESET} ", label);

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    if dim {
        for _ in 0..cw {
            let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
        }
    } else {
        for _ in 0..empty_slots {
            let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
        }
        for &val in history.iter().skip(data_len.saturating_sub(cw)) {
            let ch = sparkline_char(val);
            let color = utilization_color(val);
            let _ = write!(buf, "{color}{ch}{COLOR_RESET}");
        }
    }

    let current_pct = history.back().copied().unwrap_or(0.0);
    let abs_w = abs_formatted.len();

    if dim {
        let _ = write!(buf, " {COLOR_DIM_GRAY}{abs_formatted}{COLOR_RESET}");
        let _ = write!(buf, "  {COLOR_DIM_GRAY}{:>3.0}%{COLOR_RESET}", current_pct);
    } else {
        let color = utilization_color(current_pct);
        let _ = write!(buf, " {color}{abs_formatted}{COLOR_RESET}");
        let _ = write!(buf, "  {color}{:>3.0}%{COLOR_RESET}", current_pct);
    }

    let used = 2 + 3 + 1 + cw + 1 + abs_w + 2 + 4;
    let pad = section_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_mem_col_inner(
    buf: &mut String,
    label: &str,
    history: &std::collections::VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    col_width: usize,
    dim: bool,
) {
    let _ = write!(buf, " ");
    let _ = write!(buf, "{COLOR_WHITE}{:<3}{COLOR_RESET} ", label);

    let data_len = history.len();
    let empty_slots = cw.saturating_sub(data_len);

    if dim {
        for _ in 0..cw {
            let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
        }
    } else {
        for _ in 0..empty_slots {
            let _ = write!(buf, "{COLOR_DIM_CHART}▁{COLOR_RESET}");
        }
        for &val in history.iter().skip(data_len.saturating_sub(cw)) {
            let ch = sparkline_char(val);
            let color = utilization_color(val);
            let _ = write!(buf, "{color}{ch}{COLOR_RESET}");
        }
    }

    let current_pct = history.back().copied().unwrap_or(0.0);
    let abs_w = abs_formatted.len();

    if dim {
        let _ = write!(buf, " {COLOR_DIM_GRAY}{abs_formatted}{COLOR_RESET}");
        let _ = write!(buf, "  {COLOR_DIM_GRAY}{:>3.0}%{COLOR_RESET}", current_pct);
    } else {
        let color = utilization_color(current_pct);
        let _ = write!(buf, " {color}{abs_formatted}{COLOR_RESET}");
        let _ = write!(buf, "  {color}{:>3.0}%{COLOR_RESET}", current_pct);
    }

    let used = 1 + 3 + 1 + cw + 1 + abs_w + 2 + 4;
    let pad = col_width.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
}

pub fn render_mem_subtitle_line(
    buf: &mut String,
    col1: usize,
    col2: usize,
    col3: usize,
) {
    let titles = [
        ("RAM Utilization", col1),
        ("Swap Utilization", col2),
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
