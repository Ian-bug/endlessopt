// Shared types used across multiple modules
use std::fmt;
#[cfg(windows)]
use windows::Win32::Foundation::HANDLE;

/// Process priority classes matching Windows priority classes
///
/// These correspond to the Windows priority class constants:
/// - IDLE: 0x00000040
/// - BELOW_NORMAL: 0x00004000
/// - NORMAL: 0x00000020
/// - ABOVE_NORMAL: 0x00008000
/// - HIGH: 0x00000080
/// - REALTIME: 0x00000100
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PriorityClass {
    Idle = 0x00000040,
    BelowNormal = 0x00004000,
    Normal = 0x00000020,
    AboveNormal = 0x00008000,
    High = 0x00000080,
    Realtime = 0x00000100,
}

impl PriorityClass {
    /// Convert from u32 to PriorityClass
    ///
    /// Returns None if the value doesn't match any known priority class
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
    ///
    /// Returns a human-readable string representation of the priority class
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

impl Default for PriorityClass {
    fn default() -> Self {
        PriorityClass::Normal
    }
}

/// Error types for EndlessOpt operations
#[derive(Debug)]
pub enum EndlessOptError {
    /// Windows API operation failed
    #[allow(dead_code)]
    WindowsApi(String),
    /// Process operation failed
    Process {
        pid: u32,
        name: Option<String>,
        operation: String,
        details: String,
    },
    /// File system operation failed
    FileSystem {
        path: String,
        operation: String,
        details: String,
    },
    /// Configuration error
    Config(String),
    /// Permission denied
    #[allow(dead_code)]
    PermissionDenied(String),
    /// Invalid input
    #[allow(dead_code)]
    InvalidInput(String),
    /// Protected process operation
    ProtectedProcess(String),
}

impl fmt::Display for EndlessOptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EndlessOptError::WindowsApi(msg) => {
                write!(f, "Windows API error: {}", msg)
            }
            EndlessOptError::Process { pid, name, operation, details } => {
                if let Some(name) = name {
                    write!(f, "Process operation failed: {} (PID: {}, name: {}): {}",
                           operation, pid, name, details)
                } else {
                    write!(f, "Process operation failed: {} (PID: {}): {}",
                           operation, pid, details)
                }
            }
            EndlessOptError::FileSystem { path, operation, details } => {
                write!(f, "File system error: {} (path: {}): {}", operation, path, details)
            }
            EndlessOptError::Config(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            EndlessOptError::PermissionDenied(msg) => {
                write!(f, "Permission denied: {}", msg)
            }
            EndlessOptError::InvalidInput(msg) => {
                write!(f, "Invalid input: {}", msg)
            }
            EndlessOptError::ProtectedProcess(msg) => {
                write!(f, "Protected process: {}", msg)
            }
        }
    }
}

impl std::error::Error for EndlessOptError {}

/// Convenience type aliases for Result types (public for use across modules)
pub type MemoryResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type ProcessResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type CleanResult<T> = Result<T, Box<dyn std::error::Error>>;
pub type ConfigResult<T> = Result<T, Box<dyn std::error::Error>>;

/// RAII wrapper for Windows HANDLE to ensure proper cleanup
///
/// This wrapper automatically closes the handle when it goes out of scope,
/// preventing handle leaks even when errors occur.
#[cfg(windows)]
pub struct SafeHandle(HANDLE);

#[cfg(windows)]
impl SafeHandle {
    /// Create a new SafeHandle from a Windows HANDLE
    ///
    /// # Safety
    /// The handle must be valid and owned by this wrapper
    pub unsafe fn new(handle: HANDLE) -> Self {
        SafeHandle(handle)
    }

    /// Get the underlying HANDLE
    #[allow(dead_code)]
    pub fn as_raw(&self) -> HANDLE {
        self.0
    }
}

#[cfg(windows)]
impl Drop for SafeHandle {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;
        if self.0 != HANDLE::default() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

#[cfg(test)]
mod common_tests {
    use super::*;

    #[test]
    fn test_priority_from_u32() {
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
    fn test_priority_default() {
        assert_eq!(PriorityClass::default(), PriorityClass::Normal);
    }

    #[test]
    fn test_error_display() {
        let err = EndlessOptError::PermissionDenied("Administrator access required".to_string());
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("Administrator"));

        let err = EndlessOptError::ProtectedProcess("system.exe".to_string());
        assert!(err.to_string().contains("Protected process"));
    }
}

#[cfg(test)]
mod common_tests {
    use super::*;

    #[test]
    fn test_priority_from_u32() {
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
    fn test_priority_default() {
        assert_eq!(PriorityClass::default(), PriorityClass::Normal);
    }

    #[test]
    fn test_error_display() {
        let err = EndlessOptError::PermissionDenied("Administrator access required".to_string());
        assert!(err.to_string().contains("Permission denied"));
        assert!(err.to_string().contains("Administrator"));

        let err = EndlessOptError::ProtectedProcess("system.exe".to_string());
        assert!(err.to_string().contains("Protected process"));
    }
}
