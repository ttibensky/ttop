use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use ttop::cpu::{CpuState, TempState};
use ttop::disk::{DiskIoState, DiskSpaceState};
use ttop::gpu::GpuState;
use ttop::memory::{MemState, MemTempState};
use ttop::ui::{
    core_columns, disk_io_chart_width, disk_space_chart_width, gpu_abs_width, label_width,
    mem_abs_width, mem_col_chart_width, mem_temp_label_width, render_frame, temp_chart_width,
    temp_label_width, util_chart_width,
};

const TICK_INTERVAL: Duration = Duration::from_secs(1);

struct TerminalGuard;

impl TerminalGuard {
    fn init() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, cursor::Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn main() -> io::Result<()> {
    let _guard = TerminalGuard::init()?;

    let mut stdout = io::stdout();
    let mut cpu = CpuState::new();
    let mut temp = TempState::new();
    let mut mem = MemState::new();
    let mut mem_temp = MemTempState::new();
    let mut gpu = GpuState::new();
    let mut disk_space = DiskSpaceState::new();
    let mut disk_io = DiskIoState::new();

    loop {
        let tick_start = Instant::now();

        let (cols, rows) = terminal::size()?;

        let core_cols = core_columns(cpu.core_count());
        let num_util_cols = core_cols.len();
        let num_borders = num_util_cols + 2;
        let available = (cols as usize).saturating_sub(num_borders);
        let util_total = (available * 2) / 3;
        let temp_col = available - util_total;
        let util_sub_width = util_total / num_util_cols;

        let lw = label_width(cpu.core_count());
        let ucw = util_chart_width(util_sub_width, lw);
        cpu.update(ucw);

        let tlw = temp_label_width(&temp);
        let tcw = temp_chart_width(temp_col + 1, tlw);
        temp.update(tcw);

        let mem_avail = (cols as usize).saturating_sub(4);
        let mem_col1 = mem_avail / 3;
        let mem_col3 = mem_avail - mem_col1 - mem_avail / 3;

        let aw = mem_abs_width(&mem);
        let mcw = mem_col_chart_width(mem_col1, aw);
        mem.update(mcw);

        let mem_third_section = mem_col3 + 1;
        let mtlw = mem_temp_label_width(&mem_temp);
        let mtcw = temp_chart_width(mem_third_section, mtlw);
        mem_temp.update(mtcw);

        let gpu_avail = (cols as usize).saturating_sub(4);
        let gpu_col1 = gpu_avail / 3;
        let gpu_col2 = gpu_avail / 3;
        let gpu_col3 = gpu_avail - gpu_col1 - gpu_col2;
        let gaw = gpu_abs_width(&gpu);
        let gpu_ucw = util_chart_width(gpu_col1, 3);
        let gpu_mcw = mem_col_chart_width(gpu_col2, gaw);
        let gpu_tcw = temp_chart_width(gpu_col3 + 1, 3);
        gpu.update(gpu_ucw.max(gpu_mcw).max(gpu_tcw));

        let disk_avail = (cols as usize).saturating_sub(3);
        let disk_left = disk_avail / 2;
        let disk_right = disk_avail - disk_left;
        let disk_right_section = disk_right + 1;

        let dslw = disk_space.label_width();
        let dsaw = disk_space.abs_width();
        let dscw = disk_space_chart_width(disk_left, dslw, dsaw);
        disk_space.update(dscw);

        let dilw = disk_io.label_width();
        let dirw = disk_io.rate_width();
        let dicw = disk_io_chart_width(disk_right_section, dilw, dirw);
        disk_io.update(dicw);

        let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, &disk_space, &disk_io, cols, rows);
        stdout.write_all(frame.as_bytes())?;
        stdout.flush()?;

        let elapsed = tick_start.elapsed();
        let sleep_time = TICK_INTERVAL.saturating_sub(elapsed);

        let poll_deadline = Instant::now() + sleep_time;
        loop {
            let remaining = poll_deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }

            if event::poll(remaining.min(Duration::from_millis(100)))?
                && let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()?
            {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('Q') => return Ok(()),
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(());
                    }
                    _ => {}
                }
            }
        }
    }
}
