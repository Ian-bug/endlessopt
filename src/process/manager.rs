use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};
use sysinfo::{System, Pid};
use crate::common::{PriorityClass, EndlessOptError, ProcessResult, SafeHandle};

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

/// Get all processes currently running
///
/// Enumerates all running processes and returns detailed information.
/// Processes are sorted by name for consistent display.
///
/// # Arguments
/// - `blacklist`: Processes to mark as blacklisted (for UI purposes)
///
/// # Returns
/// - Vector of ProcessInfo with PID, name, CPU, memory, priority
///
/// # Performance
/// - Refreshes system state (typically 50-150ms)
/// - Returns 200-300 processes on typical Windows system
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
///
/// Changes the Windows priority class for a process using SetPriorityClass API.
/// Higher priority processes get more CPU time, lower priority get less.
///
/// # Arguments
/// - `pid`: Process ID
/// - `priority`: Target priority class (Idle to Realtime)
///
/// # Returns
/// - `Ok(true)`: Priority changed successfully
/// - `Err(...)`: Process not found, access denied, or API error
///
/// # Safety
/// - Requires PROCESS_SET_INFORMATION access
/// - Realtime priority can make system unresponsive if misused
/// - Handle automatically cleaned up via RAII
pub fn set_process_priority(pid: u32, priority: PriorityClass) -> ProcessResult<bool> {
    use windows::Win32::System::Threading::SetPriorityClass;
    use windows::Win32::System::Threading::PROCESS_CREATION_FLAGS;

    unsafe {
        let handle = match OpenProcess(PROCESS_SET_INFORMATION, false, pid) {
            Ok(h) => h,
            Err(e) => {
                return Err(EndlessOptError::Process {
                    pid,
                    name: None,
                    operation: "set_priority".to_string(),
                    details: format!("Failed to open process: {}", e),
                }.into())
            }
        };

        // SafeHandle will automatically close when it goes out of scope
        let _safe_handle = SafeHandle::new(handle);

        let priority_value = match priority {
            PriorityClass::Idle => 0x00000040u32,
            PriorityClass::BelowNormal => 0x00004000u32,
            PriorityClass::Normal => 0x00000020u32,
            PriorityClass::AboveNormal => 0x00008000u32,
            PriorityClass::High => 0x00000080u32,
            PriorityClass::Realtime => 0x00000100u32,
        };

        match SetPriorityClass(handle, PROCESS_CREATION_FLAGS(priority_value)) {
            Ok(_) => Ok(true),
            Err(e) => Err(EndlessOptError::Process {
                pid,
                name: None,
                operation: "set_priority".to_string(),
                details: format!("Failed to set priority: {}", e),
            }.into()),
        }
    }
}

/// Get priority for a specific process
pub fn get_process_priority(pid: u32) -> ProcessResult<PriorityClass> {
    use windows::Win32::System::Threading::GetPriorityClass;

    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_INFORMATION, false, pid)?;

        // SafeHandle will automatically close when it goes out of scope
        let _safe_handle = SafeHandle::new(handle);

        // GetPriorityClass returns the priority class value directly
        let priority_result = GetPriorityClass(handle);

        // Convert to u32 for matching
        let priority_value = priority_result as u32;

        PriorityClass::from_u32(priority_value)
            .ok_or_else(|| format!("Unknown priority class: {}", priority_value).into())
    }
}

/// Check if a process is protected and should not be killed
///
/// Protected processes include critical Windows components that should never
/// be terminated as it would cause system instability or crashes.
///
/// # Protected Categories
/// - **Core Windows**: system, registry, smss, csrss, wininit, services, lsass, winlogon
/// - **System Services**: svchost, lsm, spoolsv, sched
/// - **UI Components**: explorer, dwm, audiodg
/// - **System Apps**: sihost, taskhost, runtimebroker, dashost, systemsettingsbroker
/// - **Security**: msmpeng (Defender), securityhealthservice, wdffilevalidator
/// - **Self-Protection**: endlessopt.exe
///
/// # Arguments
/// - `name`: Process name (case-insensitive)
///
/// # Returns
/// - `true`: Process is protected, cannot be killed
/// - `false`: Process can be terminated
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
///
/// Terminates a process after checking if it's protected. This is a destructive
/// operation that cannot be undone.
///
/// # Arguments
/// - `pid`: Process ID to kill
///
/// # Returns
/// - `Ok(true)`: Process was killed
/// - `Err(...)`: Process not found, protected, or termination failed
///
/// # Safety
/// - Protected processes return error before attempting termination
/// - Critical system processes cannot be killed (26 protected processes)
/// - Use with caution - killing system processes can crash Windows
pub fn kill_process(pid: u32) -> ProcessResult<bool> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    if let Some(process) = sys.process(Pid::from_u32(pid)) {
        let name = process.name().to_string();

        // Check if process is protected
        if is_protected_process(&name) {
            return Err(EndlessOptError::ProtectedProcess(
                format!("Cannot kill protected process: {} (PID: {})", name, pid)
            ).into());
        }

        Ok(process.kill())
    } else {
        Err(EndlessOptError::Process {
            pid,
            name: None,
            operation: "kill".to_string(),
            details: "Process not found".to_string(),
        }.into())
    }
}

/// Optimize processes by setting appropriate priorities
///
/// Automatically adjusts process priorities based on whether they're games or
/// background processes. Useful for gaming or performance optimization.
///
/// # Arguments
/// - `game_processes`: List of game process names to prioritize
/// - `blacklist`: Processes to skip entirely
/// - `game_priority`: Priority for game processes (typically High)
/// - `bg_priority`: Priority for background processes (typically BelowNormal)
///
/// # Returns
/// - `Ok(OptimizeStats)`: Summary of changes made
/// - `Err(...)`: System query or API errors
///
/// # Example
/// ```rust
/// let games = vec!["minecraft.exe".to_string()];
/// let blacklist = vec!["system".to_string()];
/// let stats = optimize_processes(&games, &blacklist,
///                               PriorityClass::High,
///                               PriorityClass::BelowNormal)?;
/// ```
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
