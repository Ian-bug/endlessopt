use windows::Win32::System::ProcessStatus::EmptyWorkingSet;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_SET_INFORMATION,
};
use windows::Win32::Foundation::CloseHandle;

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, Box<dyn std::error::Error>>;

/// Clean working set for a specific process by PID
pub fn clean_process_memory(pid: u32) -> MemoryResult<bool> {
    unsafe {
        // Open process with required access rights
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_SET_INFORMATION,
            false,
            pid,
        );

        match handle {
            Ok(h) => {
                // Try to empty the working set
                let result = EmptyWorkingSet(h);
                let _ = CloseHandle(h);

                match result {
                    Ok(_) => Ok(true),
                    Err(e) => Err(format!("Failed to clean process {} memory: {}", pid, e).into()),
                }
            }
            Err(_) => Ok(false), // Process may not exist or no access
        }
    }
}

/// Clean working set for all processes (system-wide optimization)
pub fn clean_system_memory() -> MemoryResult<CleanStats> {
    use sysinfo::System;

    let mut sys = System::new_all();
    sys.refresh_processes();

    let mut cleaned = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (pid, _process) in sys.processes() {
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
    })
}

/// Clean working set for current process only
pub fn clean_current_process() -> MemoryResult<bool> {
    unsafe {
        // GetCurrentProcess() returns a pseudo handle (-1)
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
        format!(
            "Processed: {} | Cleaned: {} | Failed: {} | Skipped: {}",
            self.total_processed, self.cleaned, self.failed, self.skipped
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_current_process() {
        // This should always succeed for the current process
        let result = clean_current_process();
        assert!(result.is_ok());
    }

    #[test]
    fn test_clean_stats() {
        let stats = CleanStats {
            total_processed: 100,
            cleaned: 80,
            failed: 10,
            skipped: 10,
        };

        assert_eq!(stats.success_rate(), 80.0);
        assert!(stats.summary().contains("80"));
    }
}
