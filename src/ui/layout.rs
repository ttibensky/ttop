use crate::cpu::temperature::TempState;
use crate::gpu::GpuState;
use crate::memory::temperature::MemTempState;
use crate::memory::{max_mem_pair_width, MemState};

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

/// Compute the maximum width of the absolute value text (e.g. `5.1GB/8.0GB`)
/// for both RAM and SWP across all possible used values, so the chart width
/// stays stable as usage fluctuates across unit boundaries.
pub fn mem_abs_width(mem: &MemState) -> usize {
    max_mem_pair_width(mem.current.mem_total_kb)
        .max(max_mem_pair_width(mem.current.swap_total_kb))
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

/// Compute the maximum width of the absolute value text for GPU memory across
/// all possible used values, so the chart width stays stable.
pub fn gpu_abs_width(gpu: &GpuState) -> usize {
    max_mem_pair_width(gpu.current_mem_total_kb)
}

/// Compute the chart width for a disk space column.
/// Layout: " " + label + " " + chart + " " + abs + "  " + "NNN%"
///          1  + lw    + 1   + cw    + 1   + aw  + 2   + 4   = lw + aw + cw + 9
pub fn disk_space_chart_width(col_width: usize, lw: usize, aw: usize) -> usize {
    let fixed = lw + aw + 9;
    if col_width > fixed {
        col_width - fixed
    } else {
        8
    }
}

/// Compute the chart width for a disk I/O column (right half with trailing │).
/// Layout: " " + label + " " + chart + " " + rate + " │"
///          1  + lw    + 1   + cw    + 1   + rw   + 2   = lw + rw + cw + 5
pub fn disk_io_chart_width(col_with_border: usize, lw: usize, rw: usize) -> usize {
    let fixed = lw + rw + 5;
    if col_with_border > fixed {
        col_with_border - fixed
    } else {
        8
    }
}
