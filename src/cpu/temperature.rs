use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

const KNOWN_CPU_DRIVERS: &[&str] = &["k10temp", "coretemp"];

pub struct TempSensor {
    input_path: PathBuf,
    pub label: String,
}

pub struct TempState {
    sensors: Vec<TempSensor>,
    pub histories: Vec<VecDeque<f64>>,
}

pub fn celsius_to_fahrenheit(c: f64) -> f64 {
    c * 9.0 / 5.0 + 32.0
}

fn discover_sensors() -> Vec<TempSensor> {
    let hwmon_base = PathBuf::from("/sys/class/hwmon");
    let entries = match fs::read_dir(&hwmon_base) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        let name = match fs::read_to_string(&name_path) {
            Ok(n) => n.trim().to_string(),
            Err(_) => continue,
        };

        if !KNOWN_CPU_DRIVERS.iter().any(|&d| d == name) {
            continue;
        }

        let mut sensors = Vec::new();
        for i in 1..=128 {
            let input = path.join(format!("temp{i}_input"));
            if !input.exists() {
                break;
            }

            let label_path = path.join(format!("temp{i}_label"));
            let label = fs::read_to_string(&label_path)
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|_| format!("Sensor {i}"));

            sensors.push(TempSensor { input_path: input, label });
        }

        if !sensors.is_empty() {
            return sensors;
        }
    }

    Vec::new()
}

fn read_temp(sensor: &TempSensor) -> Option<f64> {
    fs::read_to_string(&sensor.input_path)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .map(|millideg| millideg as f64 / 1000.0)
}

impl TempState {
    pub fn new() -> Self {
        let sensors = discover_sensors();
        let count = sensors.len();
        Self {
            sensors,
            histories: vec![VecDeque::new(); count],
        }
    }

    pub fn update(&mut self, chart_width: usize) {
        for (i, sensor) in self.sensors.iter().enumerate() {
            if let Some(temp) = read_temp(sensor) {
                self.histories[i].push_back(temp);
                while self.histories[i].len() > chart_width {
                    self.histories[i].pop_front();
                }
            }
        }
    }

    pub fn sensor_count(&self) -> usize {
        self.sensors.len()
    }

    pub fn available(&self) -> bool {
        !self.sensors.is_empty()
    }

    pub fn labels(&self) -> Vec<&str> {
        self.sensors.iter().map(|s| s.label.as_str()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn celsius_to_fahrenheit_freezing() {
        assert!((celsius_to_fahrenheit(0.0) - 32.0).abs() < 0.01);
    }

    #[test]
    fn celsius_to_fahrenheit_boiling() {
        assert!((celsius_to_fahrenheit(100.0) - 212.0).abs() < 0.01);
    }

    #[test]
    fn celsius_to_fahrenheit_body_temp() {
        assert!((celsius_to_fahrenheit(37.0) - 98.6).abs() < 0.01);
    }

    #[test]
    fn celsius_to_fahrenheit_typical_cpu_temp() {
        let f = celsius_to_fahrenheit(46.0);
        assert!((f - 114.8).abs() < 0.1);
    }

    #[test]
    fn temp_state_new_does_not_panic() {
        let _state = TempState::new();
    }

    #[test]
    fn temp_state_histories_match_sensor_count() {
        let state = TempState::new();
        assert_eq!(state.histories.len(), state.sensor_count());
    }

    #[test]
    fn temp_state_starts_with_empty_histories() {
        let state = TempState::new();
        for h in &state.histories {
            assert!(h.is_empty());
        }
    }

    #[test]
    fn temp_state_update_does_not_panic() {
        let mut state = TempState::new();
        state.update(60);
    }

    #[test]
    fn temp_state_update_adds_samples_if_sensors_exist() {
        let mut state = TempState::new();
        state.update(60);
        if state.available() {
            for (i, h) in state.histories.iter().enumerate() {
                assert_eq!(h.len(), 1, "sensor {} should have 1 sample", i);
            }
        }
    }

    #[test]
    fn temp_state_update_trims_to_chart_width() {
        let mut state = TempState::new();
        if !state.available() {
            return;
        }
        let width = 3;
        for _ in 0..10 {
            state.update(width);
        }
        for (i, h) in state.histories.iter().enumerate() {
            assert!(
                h.len() <= width,
                "sensor {} history len {} exceeds width {}",
                i,
                h.len(),
                width
            );
        }
    }

    #[test]
    fn temp_state_values_in_sane_range() {
        let mut state = TempState::new();
        if !state.available() {
            return;
        }
        state.update(60);
        for (i, h) in state.histories.iter().enumerate() {
            if let Some(&val) = h.back() {
                assert!(
                    (0.0..=150.0).contains(&val),
                    "sensor {} temp {} out of sane range",
                    i,
                    val
                );
            }
        }
    }

    #[test]
    fn temp_state_labels_match_sensor_count() {
        let state = TempState::new();
        assert_eq!(state.labels().len(), state.sensor_count());
    }

    #[test]
    fn discover_sensors_does_not_panic() {
        let _sensors = discover_sensors();
    }
}
