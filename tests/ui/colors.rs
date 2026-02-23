use ttop::ui::{
    sparkline_char, sparkline_char_temp, temperature_color, utilization_color, COLOR_GREEN,
    COLOR_ORANGE, COLOR_RED, COLOR_YELLOW, SPARKLINE_CHARS,
};

#[test]
fn sparkline_char_at_zero_percent() {
    assert_eq!(sparkline_char(0.0), '▁');
}

#[test]
fn sparkline_char_at_hundred_percent() {
    assert_eq!(sparkline_char(100.0), '█');
}

#[test]
fn sparkline_char_at_fifty_percent() {
    let ch = sparkline_char(50.0);
    assert!(
        ['▃', '▄', '▅'].contains(&ch),
        "50% should map to a mid-range block, got '{}'",
        ch
    );
}

#[test]
fn sparkline_char_boundaries() {
    assert_eq!(sparkline_char(0.0), SPARKLINE_CHARS[0]);
    assert_eq!(sparkline_char(100.0), SPARKLINE_CHARS[7]);
}

#[test]
fn sparkline_char_negative_clamps_to_lowest() {
    assert_eq!(sparkline_char(-10.0), '▁');
}

#[test]
fn sparkline_char_over_hundred_clamps_to_highest() {
    assert_eq!(sparkline_char(150.0), '█');
}

#[test]
fn sparkline_char_temp_at_30() {
    assert_eq!(sparkline_char_temp(30.0), '▁');
}

#[test]
fn sparkline_char_temp_at_100() {
    assert_eq!(sparkline_char_temp(100.0), '█');
}

#[test]
fn sparkline_char_temp_at_65() {
    let ch = sparkline_char_temp(65.0);
    assert!(
        ['▃', '▄', '▅'].contains(&ch),
        "65°C should be mid-range, got '{}'",
        ch
    );
}

#[test]
fn sparkline_char_temp_below_30_clamps() {
    assert_eq!(sparkline_char_temp(10.0), '▁');
}

#[test]
fn sparkline_char_temp_above_100_clamps() {
    assert_eq!(sparkline_char_temp(120.0), '█');
}

#[test]
fn utilization_color_green_range() {
    assert_eq!(utilization_color(0.0), COLOR_GREEN);
    assert_eq!(utilization_color(12.0), COLOR_GREEN);
    assert_eq!(utilization_color(25.0), COLOR_GREEN);
}

#[test]
fn utilization_color_yellow_range() {
    assert_eq!(utilization_color(26.0), COLOR_YELLOW);
    assert_eq!(utilization_color(38.0), COLOR_YELLOW);
    assert_eq!(utilization_color(50.0), COLOR_YELLOW);
}

#[test]
fn utilization_color_orange_range() {
    assert_eq!(utilization_color(51.0), COLOR_ORANGE);
    assert_eq!(utilization_color(63.0), COLOR_ORANGE);
    assert_eq!(utilization_color(75.0), COLOR_ORANGE);
}

#[test]
fn utilization_color_red_range() {
    assert_eq!(utilization_color(76.0), COLOR_RED);
    assert_eq!(utilization_color(90.0), COLOR_RED);
    assert_eq!(utilization_color(100.0), COLOR_RED);
}

#[test]
fn temperature_color_green_range() {
    assert_eq!(temperature_color(30.0), COLOR_GREEN);
    assert_eq!(temperature_color(49.0), COLOR_GREEN);
}

#[test]
fn temperature_color_yellow_range() {
    assert_eq!(temperature_color(50.0), COLOR_YELLOW);
    assert_eq!(temperature_color(69.0), COLOR_YELLOW);
}

#[test]
fn temperature_color_orange_range() {
    assert_eq!(temperature_color(70.0), COLOR_ORANGE);
    assert_eq!(temperature_color(84.0), COLOR_ORANGE);
}

#[test]
fn temperature_color_red_range() {
    assert_eq!(temperature_color(85.0), COLOR_RED);
    assert_eq!(temperature_color(100.0), COLOR_RED);
}
