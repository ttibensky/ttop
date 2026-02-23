pub const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_ORANGE: &str = "\x1b[38;5;208m";
pub const COLOR_RED: &str = "\x1b[31m";
pub(crate) const COLOR_DIM_GRAY: &str = "\x1b[90m";
pub(crate) const COLOR_DIM_CHART: &str = "\x1b[38;5;240m";
pub(crate) const COLOR_BOLD_CYAN: &str = "\x1b[1;36m";
pub(crate) const COLOR_WHITE: &str = "\x1b[37m";
pub(crate) const COLOR_RESET: &str = "\x1b[0m";

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

/// Map a value in [0, max] to one of the 8 sparkline characters.
/// When max is zero or negative, returns the lowest character.
pub fn sparkline_char_scaled(value: f64, max: f64) -> char {
    if max <= 0.0 {
        return SPARKLINE_CHARS[0];
    }
    let ratio = (value / max).clamp(0.0, 1.0);
    let index = (ratio * 7.0).round() as usize;
    SPARKLINE_CHARS[index.min(7)]
}

/// Color for I/O throughput relative to max observed value.
/// green 0-30%, yellow 31-60%, orange 61-80%, red 81-100%.
pub fn io_color(value: f64, max: f64) -> &'static str {
    if max <= 0.0 {
        return COLOR_GREEN;
    }
    let pct = (value / max * 100.0) as u32;
    match pct {
        0..=30 => COLOR_GREEN,
        31..=60 => COLOR_YELLOW,
        61..=80 => COLOR_ORANGE,
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
