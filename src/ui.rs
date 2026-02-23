use std::collections::VecDeque;
use std::fmt::Write;

use crate::cpu::temperature::{self, TempState};
use crate::cpu::utilization::CpuState;
use crate::gpu::GpuState;
use crate::memory::{format_mem_pair, MemState};

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

/// Compute the chart width for a utilization column.
/// Layout: " " + label + " " + chart + " " + "NNN%" + padding(2)
///          1  + lw    + 1   + cw    + 1   + 4      + 2  = lw + cw + 9
pub fn util_chart_width(col_width: usize, lw: usize) -> usize {
    let fixed = lw + 9;
    if col_width > fixed {
        col_width - fixed
    } else {
        8
    }
}

/// Compute the chart width for the temperature column.
/// Layout: " " + label + " " + chart + " NNN°C (NNN°F)" + " │"
///          1  + tlw   + 1   + cw    + 14                + 2  = tlw + cw + 18
pub fn temp_chart_width(col_with_border: usize, tlw: usize) -> usize {
    let fixed = tlw + 18;
    if col_with_border > fixed {
        col_with_border - fixed
    } else {
        8
    }
}

/// Compute the width of the absolute value text (e.g. `5.1GB/8.0GB`) for both
/// RAM and SWP, returning the max so the two rows align.
pub fn mem_abs_width(mem: &MemState) -> usize {
    let ram_used = mem.current.mem_total_kb.saturating_sub(mem.current.mem_available_kb);
    let ram_text = format_mem_pair(ram_used, mem.current.mem_total_kb);

    let swap_used = mem.current.swap_total_kb.saturating_sub(mem.current.swap_free_kb);
    let swap_text = format_mem_pair(swap_used, mem.current.swap_total_kb);

    ram_text.len().max(swap_text.len())
}

/// Compute the chart width for a full-width memory row.
/// `inner_cols` is the space between the two `│` border characters (cols − 2).
/// Inner layout: " " + label(3) + " " + chart + " " + abs + "  " + "NNN%" + " "
///                1   + 3        + 1   + cw    + 1   + aw  + 2    + 4      + 1  = 13 + aw + cw
pub fn mem_chart_width(inner_cols: usize, abs_w: usize) -> usize {
    let fixed = 13 + abs_w;
    if inner_cols > fixed {
        inner_cols - fixed
    } else {
        8
    }
}

/// Compute the width of the absolute value text for GPU memory.
pub fn gpu_abs_width(gpu: &GpuState) -> usize {
    let text = format_mem_pair(gpu.current_mem_used_kb, gpu.current_mem_total_kb);
    text.len()
}

/// Compute the chart width for a full-width GPU row.
/// Must accommodate MEM row (same as memory: 13 + abs_w) and TMP row (20 fixed).
pub fn gpu_chart_width(inner_cols: usize, abs_w: usize) -> usize {
    let mem_fixed = 13 + abs_w;
    let temp_fixed = 20;
    let fixed = mem_fixed.max(temp_fixed);
    if inner_cols > fixed {
        inner_cols - fixed
    } else {
        8
    }
}

fn render_gpu_util_row(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    inner: usize,
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
    let pad = inner.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

fn render_gpu_mem_row(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    inner: usize,
) {
    let current_pct = history.back().copied().unwrap_or(0.0);
    let color = utilization_color(current_pct);

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
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

    let used = 2 + 3 + 1 + cw + 1 + abs_w + 2 + 4;
    let pad = inner.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

fn render_gpu_temp_row(
    buf: &mut String,
    history: &VecDeque<f64>,
    cw: usize,
    inner: usize,
) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET} ");
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

    // 2 + 3 + 1 + cw + 14 = 20 + cw
    let used = 20 + cw;
    let pad = inner.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
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

fn render_util_row_inner(
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

fn render_empty_first_col(buf: &mut String, section_width: usize) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    let inner = section_width.saturating_sub(1);
    for _ in 0..inner {
        buf.push(' ');
    }
}

fn render_empty_col(buf: &mut String, col_width: usize) {
    for _ in 0..col_width {
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

fn render_mem_row(
    buf: &mut String,
    label: &str,
    history: &VecDeque<f64>,
    cw: usize,
    abs_formatted: &str,
    inner: usize,
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
    let pad = inner.saturating_sub(used);
    for _ in 0..pad {
        buf.push(' ');
    }
    let _ = write!(buf, " {COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

fn render_mem_separator(buf: &mut String, cols: u16) {
    let inner = (cols as usize).saturating_sub(2);
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for _ in 0..inner {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
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
    util_title: &str,
    temp_title: &str,
    util_col1: usize,
    util_col2: usize,
    temp_col: usize,
) {
    let util_span = util_col1 + 1 + util_col2;
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let util_pad = util_span.saturating_sub(util_title.len()) / 2;
    let util_remaining = util_span.saturating_sub(util_pad + util_title.len());
    for _ in 0..util_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{util_title}{COLOR_RESET}");
    for _ in 0..util_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

    let temp_pad = temp_col.saturating_sub(temp_title.len()) / 2;
    let temp_remaining = temp_col.saturating_sub(temp_pad + temp_title.len());
    for _ in 0..temp_pad {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_BOLD_CYAN}{temp_title}{COLOR_RESET}");
    for _ in 0..temp_remaining {
        buf.push(' ');
    }

    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

fn render_separator_line(
    buf: &mut String,
    util_col1: usize,
    util_col2: usize,
    temp_col: usize,
) {
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for _ in 0..util_col1 {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for _ in 0..util_col2 {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
    for _ in 0..temp_col {
        buf.push(' ');
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

pub fn render_frame(cpu: &CpuState, temp: &TempState, mem: &MemState, gpu: &GpuState, cols: u16, rows: u16) -> String {
    // 3-column layout: │ util_col1 │ util_col2 │ temp_col │
    let available = (cols as usize).saturating_sub(4);
    let util_total = (available * 2) / 3;
    let temp_col = available - util_total;
    let util_col1 = util_total / 2;
    let util_col2 = util_total - util_col1;

    let first_section = util_col1 + 1; // includes left │
    let third_section = temp_col + 1; // includes right │

    let lw = label_width(cpu.core_count());
    let ucw = util_chart_width(util_col1, lw);

    let tlw = temp_label_width(temp);
    let tcw = temp_chart_width(third_section, tlw);

    let core_count = cpu.core_count();
    let half = core_count.div_ceil(2);
    let temp_rows = if temp.available() {
        temp.sensor_count()
    } else {
        1
    };
    let row_count = half.max(temp_rows);

    let mut buf = String::with_capacity((cols as usize) * (rows as usize));
    buf.push_str("\x1b[H");

    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("CPU"));
    render_subtitle_line(&mut buf, "Utilization", "Temperature", util_col1, util_col2, temp_col);

    for i in 0..row_count {
        // First column: cores 0..half
        if i < half {
            let label = format!("#{}", i);
            render_util_row(&mut buf, &label, lw, &cpu.histories[i], ucw, first_section);
        } else {
            render_empty_first_col(&mut buf, first_section);
        }

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        // Second column: cores half..core_count
        let second_idx = half + i;
        if second_idx < core_count {
            let label = format!("#{}", second_idx);
            render_util_row_inner(&mut buf, &label, lw, &cpu.histories[second_idx], ucw, util_col2);
        } else {
            render_empty_col(&mut buf, util_col2);
        }

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        // Third column: temperature
        if i < temp_rows {
            if temp.available() {
                let labels = temp.labels();
                render_temp_row(&mut buf, labels[i], tlw, &temp.histories[i], tcw, third_section);
            } else {
                render_na_temp_row(&mut buf, tlw, tcw, third_section);
            }
        } else {
            render_empty_right_half(&mut buf, third_section);
        }

        buf.push_str("\r\n");
    }

    render_separator_line(&mut buf, util_col1, util_col2, temp_col);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // --- Memory section ---
    let total_inner = (cols as usize).saturating_sub(2);
    let aw = mem_abs_width(mem);
    let mcw = mem_chart_width(total_inner, aw);

    let ram_used_kb = mem.current.mem_total_kb.saturating_sub(mem.current.mem_available_kb);
    let ram_abs = format!("{:>width$}", format_mem_pair(ram_used_kb, mem.current.mem_total_kb), width = aw);

    let swap_used_kb = mem.current.swap_total_kb.saturating_sub(mem.current.swap_free_kb);
    let swap_abs = format!("{:>width$}", format_mem_pair(swap_used_kb, mem.current.swap_total_kb), width = aw);

    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("Memory"));
    render_mem_separator(&mut buf, cols);
    render_mem_row(&mut buf, "RAM", &mem.ram_history, mcw, &ram_abs, total_inner, false);
    render_mem_row(
        &mut buf,
        "SWP",
        &mem.swap_history,
        mcw,
        &swap_abs,
        total_inner,
        !mem.swap_available(),
    );
    render_mem_separator(&mut buf, cols);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // --- GPU section (only when a GPU is detected) ---
    let gpu_lines = if gpu.available() {
        let gaw = gpu_abs_width(gpu);
        let gcw = gpu_chart_width(total_inner, gaw);

        let mem_used_kb = gpu.current_mem_used_kb;
        let mem_total_kb = gpu.current_mem_total_kb;
        let gpu_mem_abs = format!(
            "{:>width$}",
            format_mem_pair(mem_used_kb, mem_total_kb),
            width = gaw
        );

        let title = format!("GPU: {}", gpu.name);
        render_horizontal_border(&mut buf, '╭', '╮', cols, Some(&title));
        render_mem_separator(&mut buf, cols);
        render_gpu_util_row(&mut buf, &gpu.util_history, gcw, total_inner);
        render_gpu_mem_row(&mut buf, &gpu.mem_history, gcw, &gpu_mem_abs, total_inner);
        if gpu.has_temperature() {
            render_gpu_temp_row(&mut buf, &gpu.temp_history, gcw, total_inner);
            render_mem_separator(&mut buf, cols);
            render_horizontal_border(&mut buf, '╰', '╯', cols, None);
            7
        } else {
            render_mem_separator(&mut buf, cols);
            render_horizontal_border(&mut buf, '╰', '╯', cols, None);
            6
        }
    } else {
        0
    };

    // fill remaining rows
    // CPU: 4 border/subtitle/separator lines + row_count data rows
    // Memory: 4 border/separator lines + 2 data rows
    let used_lines = (row_count + 4) + 6 + gpu_lines;
    let remaining_lines = (rows as usize).saturating_sub(used_lines + 1);
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
