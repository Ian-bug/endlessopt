// Input validation utilities for EndlessOpt
use std::path::Path;
use crate::common::EndlessOptError;

/// Validate a process name
///
/// Process names should:
/// - Not be empty
/// - Not contain invalid characters
/// - Not exceed Windows MAX_PATH (260 characters)
/// - Not contain path separators
pub fn validate_process_name(name: &str) -> Result<(), EndlessOptError> {
    // Check for empty string
    if name.trim().is_empty() {
        return Err(EndlessOptError::InvalidInput(
            "Process name cannot be empty".to_string()
        ));
    }

    // Check length (Windows MAX_PATH is 260)
    if name.len() > 260 {
        return Err(EndlessOptError::InvalidInput(
            format!("Process name too long: {} characters (max 260)", name.len())
        ));
    }

    // Check for path separators (process names shouldn't have paths)
    if name.contains('/') || name.contains('\\') {
        return Err(EndlessOptError::InvalidInput(
            "Process name cannot contain path separators".to_string()
        ));
    }

    // Check for obviously invalid characters
    let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
    if name.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(EndlessOptError::InvalidInput(
            format!("Process name contains invalid characters: {}", name)
        ));
    }

    Ok(())
}

/// Validate a file path
///
/// File paths should:
/// - Not be empty
/// - Have a valid format
/// - Not exceed system limits
pub fn validate_file_path(path: &str) -> Result<(), EndlessOptError> {
    if path.trim().is_empty() {
        return Err(EndlessOptError::InvalidInput(
            "File path cannot be empty".to_string()
        ));
    }

    // Check if path looks reasonable (basic validation)
    let path_obj = Path::new(path);

    // Check for obviously invalid characters in path
    if let Some(filename) = path_obj.file_name() {
        if let Some(name_str) = filename.to_str() {
            let invalid_chars = ['<', '>', ':', '"', '|', '?', '*'];
            if name_str.chars().any(|c| invalid_chars.contains(&c)) {
                return Err(EndlessOptError::InvalidInput(
                    format!("Path contains invalid characters: {}", name_str)
                ));
            }
        }
    }

    Ok(())
}

/// Validate a PID (Process ID)
///
/// Valid PIDs should be:
/// - Greater than 0
/// - Within reasonable range (typically < 1,000,000 on Windows)
pub fn validate_pid(pid: u32) -> Result<(), EndlessOptError> {
    if pid == 0 {
        return Err(EndlessOptError::InvalidInput(
            "PID cannot be 0 (System Idle Process)".to_string()
        ));
    }

    if pid > 10_000_000 {
        return Err(EndlessOptError::InvalidInput(
            format!("PID {} is unreasonably large (max ~10,000,000)", pid)
        ));
    }

    Ok(())
}

/// Validate configuration values
pub fn validate_config(
    game_processes: &[String],
    blacklisted_processes: &[String],
    auto_interval: u64,
) -> Result<(), EndlessOptError> {
    // Validate game process names
    for process in game_processes {
        validate_process_name(process)?;
    }

    // Validate blacklisted process names
    for process in blacklisted_processes {
        validate_process_name(process)?;
    }

    // Validate auto-optimization interval (1 minute to 24 hours)
    if auto_interval < 1 {
        return Err(EndlessOptError::InvalidInput(
            "Auto-optimization interval must be at least 1 minute".to_string()
        ));
    }

    if auto_interval > 1440 {
        return Err(EndlessOptError::InvalidInput(
            "Auto-optimization interval cannot exceed 1440 minutes (24 hours)".to_string()
        ));
    }

    // Check for duplicates between game and blacklist
    for game in game_processes {
        if blacklisted_processes.iter()
            .any(|blacklisted| blacklisted.eq_ignore_ascii_case(game)) {
            return Err(EndlessOptError::InvalidInput(
                format!("Process {} cannot be both a game and blacklisted", game)
            ));
        }
    }

    Ok(())
}

/// Sanitize a process name input
///
/// Removes leading/trailing whitespace and converts to lowercase for comparison
pub fn sanitize_process_name(name: &str) -> String {
    name.trim().to_lowercase()
}

/// Check if a string looks like a valid Windows executable name
pub fn is_valid_executable_name(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    // Common executable extensions
    let valid_extensions = [
        ".exe", ".com", ".bat", ".cmd", ".scr",
        ".msi", ".app", ".appx"
    ];

    // Must have an extension
    if !valid_extensions.iter().any(|ext| name_lower.ends_with(ext)) {
        return false;
    }

    // Must pass basic validation
    validate_process_name(name).is_ok()
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    #[test]
    fn test_validate_process_name_valid() {
        assert!(validate_process_name("notepad.exe").is_ok());
        assert!(validate_process_name("chrome.exe").is_ok());
        assert!(validate_process_name("MyApp.exe").is_ok());
    }

    #[test]
    fn test_validate_process_name_invalid() {
        assert!(validate_process_name("").is_err());
        assert!(validate_process_name("   ").is_err());
        assert!(validate_process_name("test.exe").is_ok());
        assert!(validate_process_name("test/exe").is_err());
        assert!(validate_process_name("test*.exe").is_err());
    }

    #[test]
    fn test_validate_pid() {
        assert!(validate_pid(1).is_ok());
        assert!(validate_pid(1000).is_ok());
        assert!(validate_pid(0).is_err());
        assert!(validate_pid(10_000_001).is_err());
    }

    #[test]
    fn test_sanitize_process_name() {
        assert_eq!(sanitize_process_name("  Notepad.EXE  "), "notepad.exe");
        assert_eq!(sanitize_process_name("CHROME.EXE"), "chrome.exe");
    }

    #[test]
    fn test_is_valid_executable_name() {
        assert!(is_valid_executable_name("notepad.exe"));
        assert!(is_valid_executable_name("chrome.exe"));
        assert!(!is_valid_executable_name("notepad"));
        assert!(!is_valid_executable_name("chrome.txt"));
    }

    #[test]
    fn test_validate_config() {
        // Valid config
        assert!(validate_config(
            &["game.exe".to_string()],
            &["system.exe".to_string()],
            30
        ).is_ok());

        // Duplicate in game and blacklist
        assert!(validate_config(
            &["game.exe".to_string()],
            &["game.exe".to_string()],
            30
        ).is_err());

        // Invalid interval
        assert!(validate_config(
            &["game.exe".to_string()],
            &["system.exe".to_string()],
            0
        ).is_err());
    }
}
