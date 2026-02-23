use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;

pub struct MemTempSensor {
    input_path: PathBuf,
    pub label: String,
}

pub struct MemTempState {
    sensors: Vec<MemTempSensor>,
    pub histories: Vec<VecDeque<f64>>,
}

pub fn discover_sensors() -> Vec<MemTempSensor> {
    let hwmon_base = PathBuf::from("/sys/class/hwmon");
    let entries = match fs::read_dir(&hwmon_base) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut sensors = Vec::new();
    let mut dimm_index = 0usize;

    for entry in entries.flatten() {
        let path = entry.path();
        let name_path = path.join("name");
        let name = match fs::read_to_string(&name_path) {
            Ok(n) => n.trim().to_string(),
            Err(_) => continue,
        };

        if name != "jc42" {
            continue;
        }

        let input = path.join("temp1_input");
        if !input.exists() {
            continue;
        }

        let label = fs::read_to_string(path.join("temp1_label"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| format!("DIMM{dimm_index}"));

        sensors.push(MemTempSensor {
            input_path: input,
            label,
        });
        dimm_index += 1;
    }

    sensors
}

fn read_temp(sensor: &MemTempSensor) -> Option<f64> {
    fs::read_to_string(&sensor.input_path)
        .ok()
        .and_then(|s| s.trim().parse::<i64>().ok())
        .map(|millideg| millideg as f64 / 1000.0)
}

impl Default for MemTempState {
    fn default() -> Self {
        Self::new()
    }
}

impl MemTempState {
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
