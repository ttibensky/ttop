mod cpu;
mod ui;

use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};

use cpu::CpuState;
use ui::{chart_width, label_width, render_frame};

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

    loop {
        let tick_start = Instant::now();

        let (cols, rows) = terminal::size()?;
        let lw = label_width(cpu.core_count());
        let cw = chart_width(cols, lw);
        cpu.update(cw);

        let frame = render_frame(&cpu, cols, rows);
        stdout.write_all(frame.as_bytes())?;
        stdout.flush()?;

        let elapsed = tick_start.elapsed();
        let sleep_time = TICK_INTERVAL.saturating_sub(elapsed);

        // Poll for key events during the remaining sleep time, in small chunks
        // so we stay responsive to quit commands
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
