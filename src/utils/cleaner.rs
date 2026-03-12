use std::fs;
use std::path::{Path, PathBuf};
use crate::common::{CleanResult, EndlessOptError};

/// Get all temporary directories on the system
pub fn get_temp_directories() -> Vec<PathBuf> {
    let mut temp_dirs = Vec::new();

    // Windows temp directories
    if let Ok(temp) = std::env::var("TEMP") {
        temp_dirs.push(PathBuf::from(temp));
    }

    if let Ok(tmp) = std::env::var("TMP") {
        temp_dirs.push(PathBuf::from(tmp));
    }

    // User temp directory
    if let Ok(user_profile) = std::env::var("USERPROFILE") {
        let user_temp = PathBuf::from(user_profile).join("AppData").join("Local").join("Temp");
        temp_dirs.push(user_temp);
    }

    // Windows temp directory
    if let Ok(windir) = std::env::var("WINDIR") {
        let windows_temp = PathBuf::from(windir).join("Temp");
        temp_dirs.push(windows_temp);
    }

    // Prefetch directory
    if let Ok(windir) = std::env::var("WINDIR") {
        let prefetch = PathBuf::from(windir).join("Prefetch");
        temp_dirs.push(prefetch);
    }

    temp_dirs
}

/// Clean temporary files from a specific directory
pub fn clean_temp_directory(path: &Path) -> CleanResult<CleanStats> {
    let mut files_deleted = 0;
    let mut dirs_deleted = 0;
    let mut bytes_freed = 0;
    let mut errors = Vec::new();

    if !path.exists() {
        return Ok(CleanStats {
            files_deleted: 0,
            directories_deleted: 0,
            bytes_freed: 0,
            errors: vec![],
        });
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(EndlessOptError::FileSystem {
                path: path.display().to_string(),
                operation: "read_directory".to_string(),
                details: e.to_string(),
            }.into())
        }
    };

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if metadata.is_file() {
            // Get file size before deleting
            let size = metadata.len();

            // Try to delete the file
            if fs::remove_file(&entry_path).is_ok() {
                files_deleted += 1;
                bytes_freed += size;
            } else {
                errors.push(format!("Failed to delete {}: {}",
                                   entry_path.display(),
                                   std::io::Error::last_os_error()));
            }
        } else if metadata.is_dir() {
            // Try to clean and remove the directory
            match clean_temp_directory(&entry_path) {
                Ok(stats) => {
                    files_deleted += stats.files_deleted;
                    bytes_freed += stats.bytes_freed;

                    // Try to remove the now-empty directory
                    if fs::remove_dir(&entry_path).is_ok() {
                        dirs_deleted += 1;
                    }
                }
                Err(_) => {
                    errors.push(format!("Failed to clean directory: {}", entry_path.display()));
                }
            }
        }
    }

    Ok(CleanStats {
        files_deleted,
        directories_deleted: dirs_deleted,
        bytes_freed,
        errors,
    })
}

/// Clean all temporary directories on the system
pub fn clean_temp_files() -> CleanResult<SystemCleanStats> {
    let temp_dirs = get_temp_directories();
    let mut total_stats = SystemCleanStats::default();

    for dir in temp_dirs {
        match clean_temp_directory(&dir) {
            Ok(stats) => {
                total_stats.directories_cleaned += 1;
                total_stats.total_files_deleted += stats.files_deleted;
                total_stats.total_directories_deleted += stats.directories_deleted;
                total_stats.total_bytes_freed += stats.bytes_freed;
                total_stats.errors.extend(stats.errors);
            }
            Err(e) => {
                total_stats.errors.push(format!("Failed to clean {}: {}",
                                               dir.display(), e));
            }
        }
    }

    Ok(total_stats)
}

/// Release network resources (DNS cache, etc.)
pub fn release_network_resources() -> CleanResult<NetworkStats> {
    use std::process::Command;

    let mut commands_executed = 0;
    let mut successful = 0;
    let mut errors = Vec::new();

    // Flush DNS cache
    match Command::new("ipconfig")
        .args(["/flushdns"])
        .output()
    {
        Ok(output) => {
            commands_executed += 1;
            if output.status.success() {
                successful += 1;
            } else {
                errors.push("Failed to flush DNS cache".to_string());
            }
        }
        Err(e) => {
            errors.push(format!("Failed to execute ipconfig: {}", e));
        }
    }

    // Reset network stack (optional, may require admin)
    // This is commented out as it's more invasive
    /*
    match Command::new("netsh")
        .args(&["winsock", "reset"])
        .output()
    {
        Ok(output) => {
            commands_executed += 1;
            if output.status.success() {
                successful += 1;
            } else {
                errors.push("Failed to reset winsock".to_string());
            }
        }
        Err(_) => {
            errors.push("Failed to execute netsh".to_string());
        }
    }
    */

    Ok(NetworkStats {
        commands_executed,
        successful,
        errors,
    })
}

/// Statistics from cleaning a directory
#[derive(Debug, Clone)]
pub struct CleanStats {
    pub files_deleted: usize,
    pub directories_deleted: usize,
    pub bytes_freed: u64,
    pub errors: Vec<String>,
}

/// Statistics from system-wide cleaning
#[derive(Debug, Clone, Default)]
pub struct SystemCleanStats {
    pub directories_cleaned: usize,
    pub total_files_deleted: usize,
    pub total_directories_deleted: usize,
    pub total_bytes_freed: u64,
    pub errors: Vec<String>,
}

impl SystemCleanStats {
    pub fn summary(&self) -> String {
        let size_str = if self.total_bytes_freed >= 1024 * 1024 * 1024 {
            format!("{:.2} GB", self.total_bytes_freed as f64 / (1024.0 * 1024.0 * 1024.0))
        } else if self.total_bytes_freed >= 1024 * 1024 {
            format!("{:.2} MB", self.total_bytes_freed as f64 / (1024.0 * 1024.0))
        } else if self.total_bytes_freed >= 1024 {
            format!("{:.2} KB", self.total_bytes_freed as f64 / 1024.0)
        } else {
            format!("{} B", self.total_bytes_freed)
        };

        format!(
            "Cleaned {} directories | Deleted {} files | Freed {} | Errors: {}",
            self.directories_cleaned,
            self.total_files_deleted,
            size_str,
            self.errors.len()
        )
    }
}

/// Statistics from network optimization
#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub commands_executed: usize,
    pub successful: usize,
    pub errors: Vec<String>,
}

impl NetworkStats {
    pub fn summary(&self) -> String {
        if self.errors.is_empty() {
            format!("Executed {} commands successfully", self.successful)
        } else {
            format!(
                "Executed: {} | Successful: {} | Errors: {}",
                self.commands_executed,
                self.successful,
                self.errors.len()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_temp_directories() {
        let dirs = get_temp_directories();
        assert!(!dirs.is_empty());
    }

    #[test]
    fn test_format_bytes() {
        let stats = SystemCleanStats {
            total_bytes_freed: 1024 * 1024 * 100, // 100 MB
            ..Default::default()
        };

        let summary = stats.summary();
        assert!(summary.contains("100"));
    }
}
