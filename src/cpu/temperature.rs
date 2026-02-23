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

pub fn discover_sensors() -> Vec<TempSensor> {
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

impl Default for TempState {
    fn default() -> Self {
        Self::new()
    }
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
