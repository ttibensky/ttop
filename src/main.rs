use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use ttop::cpu::{CpuState, TempState};
use ttop::gpu::GpuState;
use ttop::memory::{MemState, MemTempState};
use ttop::ui::{
    gpu_abs_width, gpu_chart_width, label_width, mem_abs_width, mem_col_chart_width,
    mem_temp_label_width, render_frame, temp_chart_width, temp_label_width, util_chart_width,
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

    loop {
        let tick_start = Instant::now();

        let (cols, rows) = terminal::size()?;

        let available = (cols as usize).saturating_sub(4);
        let util_total = (available * 2) / 3;
        let temp_col = available - util_total;
        let util_col1 = util_total / 2;

        let lw = label_width(cpu.core_count());
        let ucw = util_chart_width(util_col1, lw);
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

        let total_inner = (cols as usize).saturating_sub(2);
        let gaw = gpu_abs_width(&gpu);
        let gcw = gpu_chart_width(total_inner, gaw);
        gpu.update(gcw);

        let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, cols, rows);
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
