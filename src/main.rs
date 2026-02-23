use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use ttop::cpu::{CpuState, TempState};
use ttop::memory::MemState;
use ttop::ui::{
    label_width, left_chart_width, mem_abs_width, mem_chart_width, render_frame, right_chart_width,
    temp_label_width,
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

    loop {
        let tick_start = Instant::now();

        let (cols, rows) = terminal::size()?;

        let total_inner = (cols as usize).saturating_sub(2);
        let left_half = total_inner / 2 + 1;
        let right_half = (cols as usize).saturating_sub(left_half + 1);

        let lw = label_width(cpu.core_count());
        let lcw = left_chart_width(left_half, lw);
        cpu.update(lcw);

        let tlw = temp_label_width(&temp);
        let rcw = right_chart_width(right_half, tlw);
        temp.update(rcw);

        let aw = mem_abs_width(&mem);
        let mcw = mem_chart_width(total_inner, aw);
        mem.update(mcw);

        let frame = render_frame(&cpu, &temp, &mem, cols, rows);
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
