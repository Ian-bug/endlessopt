use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};
use windows::Win32::Foundation::CloseHandle;
use sysinfo::{Pid, System};

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Minimum memory threshold (MB) for a process to be considered for cleaning
const MIN_MEMORY_THRESHOLD_MB: u64 = 50;

/// Batch size for process cleaning to avoid system freeze
const CLEAN_BATCH_SIZE: usize = 20;

/// Memory optimization strategies inspired by PCL-CE
#[derive(Debug, Clone, Copy)]
pub enum OptimizationLevel {
    Basic,       // Just EmptyWorkingSet for large processes
    Standard,    // EmptyWorkingSet for all processes + filtering
    Aggressive,  // EmptyWorkingSet + process list refresh
}

/// Advanced memory optimization using PCL-CE inspired techniques
pub fn optimize_memory_pcl_style(level: OptimizationLevel) -> MemoryResult<OptimizationResult> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let before_available = get_available_memory_mb();

    match level {
        OptimizationLevel::Basic => {
            // Only clean large processes (>50MB)
            clean_large_processes_filtered(&[])?;
        }
        OptimizationLevel::Standard => {
            // Clean all processes with smart filtering
            clean_system_memory_filtered(&[])?;
        }
        OptimizationLevel::Aggressive => {
            // Full optimization: clean all processes multiple times
            clean_system_memory_filtered(&[])?;

            // Small delay to let system settle
            std::thread::sleep(std::time::Duration::from_millis(100));

            // Second pass for any newly allocated memory
            clean_system_memory_filtered(&[])?;
        }
    }

    // Final memory measurement
    sys.refresh_memory();
    let after_available = sys.available_memory() / 1024;
    let gained = after_available.saturating_sub(before_available);

    Ok(OptimizationResult {
        level,
        memory_before_mb: before_available,
        memory_after_mb: after_available,
        memory_gained_mb: gained,
        processes_optimized: sys.processes().len(),
    })
}

/// Get available physical memory in MB
fn get_available_memory_mb() -> u64 {
    let mut sys = System::new_all();
    sys.refresh_memory();
    sys.available_memory() / 1024
}

/// Clean only large processes (>50MB) with filtering
fn clean_large_processes_filtered(blacklist: &[String]) -> MemoryResult<CleanStats> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut cleaned = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut blacklisted = 0;
    let mut below_threshold = 0;
    let mut critical_skipped = 0;

    for (pid, process) in sys.processes() {
        let process_name = process.name().to_string();

        // Check blacklist
        if blacklist.iter().any(|name| name.eq_ignore_ascii_case(&process_name)) {
            blacklisted += 1;
            continue;
        }

        // Check memory threshold
        let memory_mb = process.memory() / 1024;
        if memory_mb < MIN_MEMORY_THRESHOLD_MB {
            below_threshold += 1;
            continue;
        }

        // Check critical processes
        if is_critical_system_process(&process_name) {
            critical_skipped += 1;
            continue;
        }

        // Clean the process
        match clean_process_memory(pid.as_u32()) {
            Ok(true) => cleaned += 1,
            Ok(false) => skipped += 1,
            Err(_) => failed += 1,
        }
    }

    Ok(CleanStats {
        total_processed: cleaned + failed + skipped + blacklisted + below_threshold + critical_skipped,
        cleaned,
        failed,
        skipped,
        blacklisted,
        below_threshold,
        critical_skipped,
    })
}

/// Clean working set for a specific process by PID
pub fn clean_process_memory(pid: u32) -> MemoryResult<bool> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION,
            false,
            pid,
        );

        match handle {
            Ok(h) => {
                let result = EmptyWorkingSet(h);
                let _ = CloseHandle(h);

                match result {
                    Ok(_) => Ok(true),
                    Err(e) => Err(format!("Failed to clean process {} memory: {}", pid, e).into()),
                }
            }
            Err(_) => Ok(false),
        }
    }
}

/// Check if a process is a critical system process
fn is_critical_system_process(name: &str) -> bool {
    let critical_processes = [
        "system", "registry", "smss.exe", "csrss.exe", "wininit.exe",
        "services.exe", "lsass.exe", "winlogon.exe", "svchost.exe",
        "lsm.exe", "dwm.exe", "audiodg.exe", "spoolsv.exe", "sched.exe",
        "systemsettingsbroker.exe", "sihost.exe", "taskhost.exe",
        "runtimebroker.exe", "dashost.exe",
    ];

    critical_processes.iter()
        .any(|proc| proc.eq_ignore_ascii_case(name))
}

/// Enhanced system-wide memory cleaning with blacklist and filtering
pub fn clean_system_memory_filtered(blacklist: &[String]) -> MemoryResult<CleanStats> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut cleaned = 0;
    let mut failed = 0;
    let mut skipped = 0;
    let mut blacklisted = 0;
    let mut below_threshold = 0;
    let mut critical_skipped = 0;

    let process_ids: Vec<u32> = sys.processes()
        .keys()
        .map(|pid| pid.as_u32())
        .collect();

    let mut batch_count = 0;

    for pid in process_ids {
        batch_count += 1;
        if batch_count % CLEAN_BATCH_SIZE == 0 {
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        sys.refresh_processes();

        let process = match sys.process(Pid::from_u32(pid)) {
            Some(p) => p,
            None => {
                skipped += 1;
                continue;
            }
        };

        let process_name = process.name().to_string();

        if blacklist.iter().any(|name| name.eq_ignore_ascii_case(&process_name)) {
            blacklisted += 1;
            continue;
        }

        let memory_mb = process.memory() / 1024;
        if memory_mb < MIN_MEMORY_THRESHOLD_MB {
            below_threshold += 1;
            continue;
        }

        if is_critical_system_process(&process_name) {
            critical_skipped += 1;
            continue;
        }

        match clean_process_memory(pid) {
            Ok(true) => cleaned += 1,
            Ok(false) => skipped += 1,
            Err(_) => failed += 1,
        }
    }

    Ok(CleanStats {
        total_processed: cleaned + failed + skipped + blacklisted + below_threshold + critical_skipped,
        cleaned,
        failed,
        skipped,
        blacklisted,
        below_threshold,
        critical_skipped,
    })
}

/// Clean working set for all processes (basic optimization)
pub fn clean_system_memory() -> MemoryResult<CleanStats> {
    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut cleaned = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (pid, _) in sys.processes() {
        match clean_process_memory(pid.as_u32()) {
            Ok(true) => cleaned += 1,
            Ok(false) => skipped += 1,
            Err(_) => failed += 1,
        }
    }

    Ok(CleanStats {
        total_processed: cleaned + failed + skipped,
        cleaned,
        failed,
        skipped,
        blacklisted: 0,
        below_threshold: 0,
        critical_skipped: 0,
    })
}

/// Clean working set for current process only
pub fn clean_current_process() -> MemoryResult<bool> {
    unsafe {
        use windows::Win32::Foundation::HANDLE;
        let handle = HANDLE(-1isize as *mut core::ffi::c_void);
        match EmptyWorkingSet(handle) {
            Ok(_) => Ok(true),
            Err(e) => Err(format!("Failed to clean current process memory: {}", e).into()),
        }
    }
}

/// Statistics from memory cleaning operation
#[derive(Debug, Clone)]
pub struct CleanStats {
    pub total_processed: usize,
    pub cleaned: usize,
    pub failed: usize,
    pub skipped: usize,
    pub blacklisted: usize,
    pub below_threshold: usize,
    pub critical_skipped: usize,
}

impl CleanStats {
    pub fn success_rate(&self) -> f32 {
        if self.total_processed == 0 {
            0.0
        } else {
            (self.cleaned as f32 / self.total_processed as f32) * 100.0
        }
    }

    pub fn summary(&self) -> String {
        let mut parts = vec![
            format!("Cleaned: {}", self.cleaned),
            format!("Failed: {}", self.failed),
        ];

        if self.blacklisted > 0 {
            parts.push(format!("Blacklisted: {}", self.blacklisted));
        }

        if self.below_threshold > 0 {
            parts.push(format!("Small: {}", self.below_threshold));
        }

        if self.critical_skipped > 0 {
            parts.push(format!("Critical: {}", self.critical_skipped));
        }

        parts.join(" | ")
    }

    pub fn detailed_summary(&self) -> String {
        format!(
            "Total: {} | Cleaned: {} | Failed: {} | Skipped: {} | Blacklisted: {} | Below Threshold: {} | Critical Skipped: {}",
            self.total_processed, self.cleaned, self.failed, self.skipped,
            self.blacklisted, self.below_threshold, self.critical_skipped
        )
    }
}

/// Result of PCL-style memory optimization
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub level: OptimizationLevel,
    pub memory_before_mb: u64,
    pub memory_after_mb: u64,
    pub memory_gained_mb: u64,
    pub processes_optimized: usize,
}

impl OptimizationResult {
    pub fn summary(&self) -> String {
        format!(
            "Level: {:?} | Before: {} MB | After: {} MB | Gained: {} MB | Processes: {}",
            self.level, self.memory_before_mb, self.memory_after_mb,
            self.memory_gained_mb, self.processes_optimized
        )
    }

    pub fn user_friendly_summary(&self) -> String {
        if self.memory_gained_mb > 0 {
            format!(
                "Memory optimization complete! Freed {} MB of memory. Current available: {} MB",
                self.memory_gained_mb, self.memory_after_mb
            )
        } else {
            format!(
                "Memory optimization complete! System already optimized. Current available: {} MB",
                self.memory_after_mb
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_current_process() {
        let result = clean_current_process();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_stats() {
        let stats = CleanStats {
            total_processed: 100,
            cleaned: 80,
            failed: 10,
            skipped: 5,
            blacklisted: 3,
            below_threshold: 1,
            critical_skipped: 1,
        };

        assert_eq!(stats.success_rate(), 80.0);
        assert!(stats.summary().contains("80"));
    }

    #[test]
    fn test_critical_system_process() {
        assert!(is_critical_system_process("system"));
        assert!(is_critical_system_process("svchost.exe"));
        assert!(is_critical_system_process("lsass.exe"));
        assert!(!is_critical_system_process("chrome.exe"));
        assert!(!is_critical_system_process("notepad.exe"));
    }

    #[test]
    fn test_memory_threshold() {
        assert_eq!(MIN_MEMORY_THRESHOLD_MB, 50);
    }

    #[test]
    fn test_optimization_levels() {
        let result_basic = optimize_memory_pcl_style(OptimizationLevel::Basic);
        assert!(result_basic.is_ok());

        let result_standard = optimize_memory_pcl_style(OptimizationLevel::Standard);
        assert!(result_standard.is_ok());
    }
}
