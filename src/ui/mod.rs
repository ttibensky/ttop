pub mod colors;
pub mod layout;

pub mod cpu;
pub mod gpu;
pub mod memory;

pub use colors::{
    sparkline_char, sparkline_char_temp, temperature_color, utilization_color, COLOR_GREEN,
    COLOR_ORANGE, COLOR_RED, COLOR_YELLOW, SPARKLINE_CHARS,
};
pub use layout::{
    core_columns, gpu_abs_width, label_width, mem_abs_width, mem_col_chart_width,
    mem_temp_label_width, temp_chart_width, temp_label_width, util_chart_width,
};

use std::fmt::Write;

use colors::*;
use cpu::*;
use gpu::*;
use memory::*;

use crate::cpu::temperature::TempState;
use crate::cpu::utilization::CpuState;
use crate::gpu::GpuState;
use crate::memory::temperature::MemTempState;
use crate::memory::{format_mem_pair, MemState};

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
    util_span: usize,
    temp_col: usize,
) {
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

fn render_separator_line(buf: &mut String, col_widths: &[usize]) {
    for &w in col_widths {
        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
        for _ in 0..w {
            buf.push(' ');
        }
    }
    let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}\r\n");
}

pub fn render_frame(cpu: &CpuState, temp: &TempState, mem: &MemState, mem_temp: &MemTempState, gpu: &GpuState, cols: u16, rows: u16) -> String {
    let core_count = cpu.core_count();
    let core_cols = core_columns(core_count);
    let num_util_cols = core_cols.len();
    let num_borders = num_util_cols + 2;
    let available = (cols as usize).saturating_sub(num_borders);
    let util_total = (available * 2) / 3;
    let temp_col = available - util_total;

    let util_widths: Vec<usize> = if num_util_cols == 3 {
        let w1 = util_total / 3;
        let w2 = util_total / 3;
        let w3 = util_total - w1 - w2;
        vec![w1, w2, w3]
    } else {
        let w1 = util_total / 2;
        let w2 = util_total - w1;
        vec![w1, w2]
    };

    let first_section = util_widths[0] + 1;
    let temp_section = temp_col + 1;

    let lw = label_width(core_count);
    let ucw = util_chart_width(util_widths[0], lw);

    let tlw = temp_label_width(temp);
    let tcw = temp_chart_width(temp_section, tlw);

    let col_starts: Vec<usize> = {
        let mut starts = vec![0];
        let mut acc = 0;
        for &c in &core_cols {
            acc += c;
            starts.push(acc);
        }
        starts
    };

    let max_cores_in_col = *core_cols.iter().max().unwrap_or(&0);
    let temp_rows = if temp.available() {
        temp.sensor_count()
    } else {
        1
    };
    let row_count = max_cores_in_col.max(temp_rows);

    let mut buf = String::with_capacity((cols as usize) * (rows as usize));
    buf.push_str("\x1b[H");

    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("CPU"));
    let util_span: usize = util_widths.iter().sum::<usize>() + (num_util_cols - 1);
    render_subtitle_line(&mut buf, "Utilization", "Temperature", util_span, temp_col);

    for i in 0..row_count {
        let idx = col_starts[0] + i;
        if idx < col_starts[1] {
            let label = format!("#{idx}");
            render_util_row(&mut buf, &label, lw, &cpu.histories[idx], ucw, first_section);
        } else {
            render_empty_first_col(&mut buf, first_section);
        }

        for c in 1..num_util_cols {
            let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
            let idx = col_starts[c] + i;
            if idx < col_starts[c + 1] {
                let label = format!("#{idx}");
                render_util_row_inner(&mut buf, &label, lw, &cpu.histories[idx], ucw, util_widths[c]);
            } else {
                render_empty_col(&mut buf, util_widths[c]);
            }
        }

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        if i < temp_rows {
            if temp.available() {
                let labels = temp.labels();
                render_temp_row(&mut buf, labels[i], tlw, &temp.histories[i], tcw, temp_section);
            } else {
                render_na_temp_row(&mut buf, tlw, tcw, temp_section);
            }
        } else {
            render_empty_right_half(&mut buf, temp_section);
        }

        buf.push_str("\r\n");
    }

    let mut all_col_widths: Vec<usize> = util_widths.clone();
    all_col_widths.push(temp_col);
    render_separator_line(&mut buf, &all_col_widths);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // --- Memory section (three-column layout) ---
    let mem_avail = (cols as usize).saturating_sub(4);
    let mem_col1 = mem_avail / 3;
    let mem_col2 = mem_avail / 3;
    let mem_col3 = mem_avail - mem_col1 - mem_col2;

    let mem_first_section = mem_col1 + 1;
    let mem_third_section = mem_col3 + 1;

    let aw = mem_abs_width(mem);
    let mcw = mem_col_chart_width(mem_col1, aw);

    let mtlw = mem_temp_label_width(mem_temp);
    let mtcw = temp_chart_width(mem_third_section, mtlw);

    let ram_used_kb = mem.current.mem_total_kb.saturating_sub(mem.current.mem_available_kb);
    let ram_abs = format!(
        "{:>width$}",
        format_mem_pair(ram_used_kb, mem.current.mem_total_kb),
        width = aw
    );

    let swap_used_kb = mem.current.swap_total_kb.saturating_sub(mem.current.swap_free_kb);
    let swap_abs = format!(
        "{:>width$}",
        format_mem_pair(swap_used_kb, mem.current.swap_total_kb),
        width = aw
    );

    let mem_temp_rows = if mem_temp.available() {
        mem_temp.sensor_count()
    } else {
        1
    };
    let mem_row_count = mem_temp_rows.max(1);

    render_horizontal_border(&mut buf, '╭', '╮', cols, Some("Memory"));
    render_mem_subtitle_line(&mut buf, mem_col1, mem_col2, mem_col3);

    for i in 0..mem_row_count {
        if i == 0 {
            render_mem_col_first(
                &mut buf,
                "RAM",
                &mem.ram_history,
                mcw,
                &ram_abs,
                mem_first_section,
                false,
            );
        } else {
            render_empty_first_col(&mut buf, mem_first_section);
        }

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        if i == 0 {
            render_mem_col_inner(
                &mut buf,
                "SWP",
                &mem.swap_history,
                mcw,
                &swap_abs,
                mem_col2,
                !mem.swap_available(),
            );
        } else {
            render_empty_col(&mut buf, mem_col2);
        }

        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");

        if i < mem_temp_rows {
            if mem_temp.available() {
                let labels = mem_temp.labels();
                render_temp_row(
                    &mut buf,
                    labels[i],
                    mtlw,
                    &mem_temp.histories[i],
                    mtcw,
                    mem_third_section,
                );
            } else {
                render_na_temp_row(&mut buf, mtlw, mtcw, mem_third_section);
            }
        } else {
            render_empty_right_half(&mut buf, mem_third_section);
        }

        buf.push_str("\r\n");
    }

    render_separator_line(&mut buf, &[mem_col1, mem_col2, mem_col3]);
    render_horizontal_border(&mut buf, '╰', '╯', cols, None);

    // --- GPU section (only when a GPU is detected) ---
    let gpu_lines = if gpu.available() {
        let gpu_avail = (cols as usize).saturating_sub(4);
        let gpu_col1 = gpu_avail / 3;
        let gpu_col2 = gpu_avail / 3;
        let gpu_col3 = gpu_avail - gpu_col1 - gpu_col2;

        let gpu_first_section = gpu_col1 + 1;
        let gpu_third_section = gpu_col3 + 1;

        let gaw = gpu_abs_width(gpu);
        let gpu_ucw = util_chart_width(gpu_col1, 3);
        let gpu_mcw = mem_col_chart_width(gpu_col2, gaw);
        let gpu_tcw = temp_chart_width(gpu_third_section, 3);

        let mem_used_kb = gpu.current_mem_used_kb;
        let mem_total_kb = gpu.current_mem_total_kb;
        let gpu_mem_abs = format!(
            "{:>width$}",
            format_mem_pair(mem_used_kb, mem_total_kb),
            width = gaw
        );

        let title = format!("GPU: {}", gpu.name);
        render_horizontal_border(&mut buf, '╭', '╮', cols, Some(&title));
        render_gpu_subtitle_line(&mut buf, gpu_col1, gpu_col2, gpu_col3);

        render_gpu_util_col_first(&mut buf, &gpu.util_history, gpu_ucw, gpu_first_section);
        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
        render_gpu_mem_col_inner(&mut buf, &gpu.mem_history, gpu_mcw, &gpu_mem_abs, gpu_col2);
        let _ = write!(buf, "{COLOR_DIM_GRAY}│{COLOR_RESET}");
        render_gpu_temp_col_right(&mut buf, &gpu.temp_history, gpu_tcw, gpu_third_section);
        buf.push_str("\r\n");

        render_separator_line(&mut buf, &[gpu_col1, gpu_col2, gpu_col3]);
        render_horizontal_border(&mut buf, '╰', '╯', cols, None);
        5
    } else {
        0
    };

    // fill remaining rows
    let mem_lines = mem_row_count + 4;
    let used_lines = (row_count + 4) + mem_lines + gpu_lines;
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
