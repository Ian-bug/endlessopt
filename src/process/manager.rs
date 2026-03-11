use windows::Win32::System::Threading::{
    OpenProcess, SetPriorityClass, GetPriorityClass,
    PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};
use windows::Win32::Foundation::CloseHandle;
use sysinfo::{System, Pid};

/// Process priority classes matching Windows priority classes
#[derive(Debug, Clone, Copy, Eq)]
pub enum PriorityClass {
    Idle = 0x00000040,
    BelowNormal = 0x00004000,
    Normal = 0x00000020,
    AboveNormal = 0x00008000,
    High = 0x00000080,
    Realtime = 0x00000100,
}

impl PartialEq for PriorityClass {
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PriorityClass {
    /// Convert from u32 to PriorityClass
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x00000040 => Some(PriorityClass::Idle),
            0x00004000 => Some(PriorityClass::BelowNormal),
            0x00000020 => Some(PriorityClass::Normal),
            0x00008000 => Some(PriorityClass::AboveNormal),
            0x00000080 => Some(PriorityClass::High),
            0x00000100 => Some(PriorityClass::Realtime),
            _ => None,
        }
    }

    /// Get display name for priority
    pub fn as_str(&self) -> &'static str {
        match self {
            PriorityClass::Idle => "Idle",
            PriorityClass::BelowNormal => "Below Normal",
            PriorityClass::Normal => "Normal",
            PriorityClass::AboveNormal => "Above Normal",
            PriorityClass::High => "High",
            PriorityClass::Realtime => "Realtime",
        }
    }
}

/// Information about a process
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub priority: PriorityClass,
    pub is_blacklisted: bool,
}

/// Result type for process operations
pub type ProcessResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Get all processes currently running
pub fn get_all_processes(blacklist: &[String]) -> ProcessResult<Vec<ProcessInfo>> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut processes = Vec::new();

    for (pid, process) in sys.processes() {
        let name = process.name().to_string();
        let is_blacklisted = blacklist.iter()
            .any(|p| p.eq_ignore_ascii_case(&name));

        // Try to get actual Windows priority
        let priority = get_process_priority(pid.as_u32())
            .unwrap_or(PriorityClass::Normal);

        processes.push(ProcessInfo {
            pid: pid.as_u32(),
            name,
            cpu_usage: process.cpu_usage(),
            memory_usage: process.memory(),
            priority,
            is_blacklisted,
        });
    }

    // Sort by name for consistent display
    processes.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(processes)
}

/// Set priority for a specific process
pub fn set_process_priority(pid: u32, priority: PriorityClass) -> ProcessResult<bool> {
    use windows::Win32::System::Threading::SetPriorityClass;
    use windows::Win32::System::Threading::PROCESS_CREATION_FLAGS;

    unsafe {
        let handle = OpenProcess(PROCESS_SET_INFORMATION, false, pid)?;

        let priority_value = match priority {
            PriorityClass::Idle => 0x00000040u32,
            PriorityClass::BelowNormal => 0x00004000u32,
            PriorityClass::Normal => 0x00000020u32,
            PriorityClass::AboveNormal => 0x00008000u32,
            PriorityClass::High => 0x00000080u32,
            PriorityClass::Realtime => 0x00000100u32,
        };

        let result = SetPriorityClass(handle, PROCESS_CREATION_FLAGS(priority_value));
        let _ = CloseHandle(handle);

        match result {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Failed to set priority for process {}: {}", pid, e).into()),
        }
    }
}

/// Get priority for a specific process
pub fn get_process_priority(pid: u32) -> ProcessResult<PriorityClass> {
    use windows::Win32::System::Threading::GetPriorityClass;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)?;

        // GetPriorityClass returns a value that can be converted to u32
        let priority_result = GetPriorityClass(handle);
        let _ = CloseHandle(handle);

        // Try to convert the result to a u32 value
        // The exact type depends on the windows crate version
        let priority_value = u32::from(priority_result);

        PriorityClass::from_u32(priority_value)
            .ok_or_else(|| format!("Unknown priority class: {}", priority_value).into())
    }
}

/// Check if a process is protected and should not be killed
pub fn is_protected_process(name: &str) -> bool {
    let protected = [
        // Critical Windows system processes
        "system",
        "system idle process",
        "registry",
        "smss.exe",
        "csrss.exe",
        "wininit.exe",
        "services.exe",
        "lsass.exe",
        "winlogon.exe",
        "svchost.exe",
        "lsm.exe",
        "explorer.exe",
        "dwm.exe",
        "audiodg.exe",
        "spoolsv.exe",
        "sched.exe",
        "systemsettingsbroker.exe",
        "sihost.exe",
        "taskhost.exe",
        "runtimebroker.exe",
        "dashost.exe",
        // Security processes
        "msmpeng.exe", // Windows Defender
        "securityhealthservice.exe",
        "wdffilevalidator.exe",
        // EndlessOpt itself
        "endlessopt.exe",
    ];

    protected.iter()
        .any(|p| p.eq_ignore_ascii_case(name))
}

/// Kill a specific process (with protection check)
pub fn kill_process(pid: u32) -> ProcessResult<bool> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        let name = process.name();

        // Check if process is protected
        if is_protected_process(name) {
            return Err(format!("Cannot kill protected process: {}", name).into());
        }

        Ok(process.kill())
    } else {
        Err(format!("Process {} not found", pid).into())
    }
}

/// Optimize processes by setting appropriate priorities
pub fn optimize_processes(
    game_processes: &[String],
    blacklist: &[String],
    game_priority: PriorityClass,
    bg_priority: PriorityClass,
) -> ProcessResult<OptimizeStats> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut stats = OptimizeStats {
        total_processed: 0,
        game_processes_optimized: 0,
        background_processes_optimized: 0,
        skipped: 0,
        failed: 0,
    };

    for (pid, process) in sys.processes() {
        let name = process.name().to_string();

        // Skip blacklisted processes
        if blacklist.iter().any(|p| p.eq_ignore_ascii_case(&name)) {
            stats.skipped += 1;
            continue;
        }

        stats.total_processed += 1;

        let target_priority = if game_processes.iter()
            .any(|p| p.eq_ignore_ascii_case(&name))
        {
            stats.game_processes_optimized += 1;
            game_priority
        } else {
            stats.background_processes_optimized += 1;
            bg_priority
        };

        match set_process_priority(pid.as_u32(), target_priority) {
            Ok(_) => {},
            Err(_) => stats.failed += 1,
        }
    }

    Ok(stats)
}

/// Statistics from process optimization
#[derive(Debug, Clone)]
pub struct OptimizeStats {
    pub total_processed: usize,
    pub game_processes_optimized: usize,
    pub background_processes_optimized: usize,
    pub skipped: usize,
    pub failed: usize,
}

impl OptimizeStats {
    pub fn summary(&self) -> String {
        format!(
            "Total: {} | Games: {} | Background: {} | Skipped: {} | Failed: {}",
            self.total_processed,
            self.game_processes_optimized,
            self.background_processes_optimized,
            self.skipped,
            self.failed
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_priority_class_conversion() {
        assert_eq!(PriorityClass::from_u32(0x00000040), Some(PriorityClass::Idle));
        assert_eq!(PriorityClass::from_u32(0x00000020), Some(PriorityClass::Normal));
        assert_eq!(PriorityClass::from_u32(0xFFFFFFFF), None);
    }

    #[test]
    fn test_priority_as_str() {
        assert_eq!(PriorityClass::High.as_str(), "High");
        assert_eq!(PriorityClass::Normal.as_str(), "Normal");
    }

    #[test]
    fn test_get_all_processes() {
        let processes = get_all_processes(&[]).unwrap();
        assert!(!processes.is_empty());
    }

    #[test]
    fn test_protected_process() {
        assert!(is_protected_process("system"));
        assert!(is_protected_process("csrss.exe"));
        assert!(is_protected_process("explorer.exe"));
        assert!(!is_protected_process("notepad.exe"));
    }
}
