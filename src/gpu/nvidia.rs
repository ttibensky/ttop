use std::process::Command;

pub struct NvidiaGpu {
    pub name: String,
}

pub struct NvidiaSnapshot {
    pub utilization_pct: f64,
    pub memory_used_mib: u64,
    pub memory_total_mib: u64,
    pub temperature_c: f64,
}

pub fn detect() -> Option<NvidiaGpu> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=name", "--format=csv,noheader,nounits"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if name.is_empty() {
        return None;
    }

    Some(NvidiaGpu { name })
}

pub fn read_snapshot() -> Option<NvidiaSnapshot> {
    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = text.trim().split(',').map(|s| s.trim()).collect();

    if parts.len() < 4 {
        return None;
    }

    Some(NvidiaSnapshot {
        utilization_pct: parts[0].parse().ok()?,
        memory_used_mib: parts[1].parse().ok()?,
        memory_total_mib: parts[2].parse().ok()?,
        temperature_c: parts[3].parse().ok()?,
    })
}
