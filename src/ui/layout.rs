use crate::cpu::temperature::TempState;
use crate::gpu::GpuState;
use crate::memory::temperature::MemTempState;
use crate::memory::{format_mem_pair, MemState};

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

/// Split cores across utilization sub-columns.
///
/// Tries a 3-column layout (ceil(n/3) per column, remainder in the last).
/// Falls back to 2 columns when the third column would be empty.
pub fn core_columns(core_count: usize) -> Vec<usize> {
    if core_count == 0 {
        return vec![0, 0];
    }
    let per = core_count.div_ceil(3);
    let col1 = per;
    let col2 = per.min(core_count.saturating_sub(per));
    let col3 = core_count.saturating_sub(col1 + col2);
    if col3 > 0 {
        vec![col1, col2, col3]
    } else {
        let half = core_count.div_ceil(2);
        vec![half, core_count.saturating_sub(half)]
    }
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

/// Compute the chart width for a memory column (RAM or SWAP) in the three-column layout.
/// `col_inner` is the inner width of the column (without borders).
/// Layout: " " + label(3) + " " + chart + " " + abs + "  " + "NNN%"
///          1   + 3        + 1   + cw    + 1   + aw  + 2    + 4     = 12 + aw + cw
pub fn mem_col_chart_width(col_inner: usize, abs_w: usize) -> usize {
    let fixed = 12 + abs_w;
    if col_inner > fixed {
        col_inner - fixed
    } else {
        8
    }
}

pub fn mem_temp_label_width(mem_temp: &MemTempState) -> usize {
    if !mem_temp.available() {
        return 3;
    }
    mem_temp
        .labels()
        .iter()
        .map(|l| l.len())
        .max()
        .unwrap_or(4)
        .max(3)
}

/// Compute the width of the absolute value text for GPU memory.
pub fn gpu_abs_width(gpu: &GpuState) -> usize {
    let text = format_mem_pair(gpu.current_mem_used_kb, gpu.current_mem_total_kb);
    text.len()
}
