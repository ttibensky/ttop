use ttop::cpu::temperature::TempState;
use ttop::cpu::utilization::CpuState;
use ttop::gpu::GpuState;
use ttop::memory::{MemState, MemTempState};
use ttop::ui::render_frame;

#[test]
fn render_frame_contains_cpu_section() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("CPU"), "frame should contain CPU header");
}

#[test]
fn render_frame_contains_subtitles() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(
        stripped.contains("Utilization"),
        "frame should contain 'Utilization' subtitle"
    );
    assert!(
        stripped.contains("Temperature"),
        "frame should contain 'Temperature' subtitle"
    );
}

#[test]
fn render_frame_contains_status_bar() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("q: quit"));
    assert!(stripped.contains("ttop v0.1"));
}

#[test]
fn render_frame_contains_all_core_labels() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    for i in 0..cpu.core_count() {
        assert!(
            stripped.contains(&format!("#{}", i)),
            "frame should contain label for core #{}",
            i
        );
    }
}

#[test]
fn render_frame_has_box_drawing_chars() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains('╭'));
    assert!(stripped.contains('╮'));
    assert!(stripped.contains('╰'));
    assert!(stripped.contains('╯'));
}

#[test]
fn render_frame_contains_vertical_separator() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    let lines: Vec<&str> = stripped.lines().collect();
    if lines.len() > 2 {
        let data_line = lines[2];
        let pipe_count = data_line.chars().filter(|&c| c == '│').count();
        assert!(
            pipe_count >= 3,
            "data row should have left border, separator, and right border"
        );
    }
}

#[test]
fn render_frame_shows_temp_or_na() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 40);
    let stripped = strip_ansi(&frame);
    let has_temp = stripped.contains("°C") && stripped.contains("°F");
    let has_na = stripped.contains("N/A");
    assert!(has_temp || has_na, "frame should show temperature or N/A");
}

#[test]
fn render_frame_contains_memory_section() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("Memory"), "frame should contain Memory header");
}

#[test]
fn render_frame_contains_ram_and_swp_labels() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(stripped.contains("RAM"), "frame should contain RAM label");
    assert!(stripped.contains("SWP"), "frame should contain SWP label");
}

#[test]
fn render_frame_memory_has_three_column_subtitles() {
    let cpu = CpuState::new();
    let temp = TempState::new();
    let mem = MemState::new();
    let mem_temp = MemTempState::new();
    let gpu = GpuState::new();
    let frame = render_frame(&cpu, &temp, &mem, &mem_temp, &gpu, 120, 60);
    let stripped = strip_ansi(&frame);
    assert!(
        stripped.contains("RAM Utilization"),
        "frame should contain 'RAM Utilization' subtitle"
    );
    assert!(
        stripped.contains("Swap Utilization"),
        "frame should contain 'Swap Utilization' subtitle"
    );
}

fn strip_ansi(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            if let Some('[') = chars.next() {
                for c2 in chars.by_ref() {
                    if c2.is_ascii_alphabetic() {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}
