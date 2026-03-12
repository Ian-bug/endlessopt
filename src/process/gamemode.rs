use crate::process::manager::{set_process_priority};
use crate::common::{PriorityClass, ProcessResult};
use crate::memory::optimizer::{clean_system_memory_filtered, CleanStats};
use sysinfo::System;

/// Game mode state and configuration
#[derive(Debug, Clone)]
pub struct GameMode {
    pub active: bool,
    pub game_processes: Vec<String>,
    pub game_priority: PriorityClass,
    pub bg_priority: PriorityClass,
    pub clean_memory: bool,
    #[allow(dead_code)]
    pub optimize_network: bool,  // Reserved for future network optimization
}

impl GameMode {
    /// Create a new game mode configuration
    #[allow(dead_code)]
    pub fn new(
        game_processes: Vec<String>,
        game_priority: PriorityClass,
        bg_priority: PriorityClass,
        clean_memory: bool,
        optimize_network: bool,
    ) -> Self {
        GameMode {
            active: false,
            game_processes,
            game_priority,
            bg_priority,
            clean_memory,
            optimize_network,
        }
    }

    /// Activate game mode optimization
    #[allow(dead_code)]
    pub fn activate(&mut self) -> ProcessResult<ActivationResult> {
        let mut sys = System::new_all();
        sys.refresh_processes();

        let mut active_games = Vec::new();
        let mut game_count = 0;
        let mut bg_count = 0;
        let mut failed_count = 0;

        // Find active game processes
        for (pid, process) in sys.processes() {
            let name = process.name().to_string();

            if self.is_game_process(&name) {
                active_games.push((pid.as_u32(), name.clone()));
                game_count += 1;

                match set_process_priority(pid.as_u32(), self.game_priority) {
                    Ok(_) => {},
                    Err(_) => failed_count += 1,
                }
            }
        }

        // Optimize background processes (lower priority)
        for (pid, process) in sys.processes() {
            let name = process.name().to_string();

            // Skip game processes
            if self.is_game_process(&name) {
                continue;
            }

            bg_count += 1;

            match set_process_priority(pid.as_u32(), self.bg_priority) {
                Ok(_) => {},
                Err(_) => failed_count += 1,
            }
        }

        // Clean memory if enabled (using filtered cleaning)
        let memory_stats = if self.clean_memory {
            Some(clean_system_memory_filtered(&[])?)
        } else {
            None
        };

        self.active = true;

        Ok(ActivationResult {
            games_detected: active_games,
            game_count,
            background_processes_optimized: bg_count,
            failed_count,
            memory_cleaned: memory_stats,
        })
    }

    /// Deactivate game mode and restore normal priorities
    #[allow(dead_code)]
    pub fn deactivate(&mut self) -> ProcessResult<DeactivationResult> {
        let mut sys = System::new_all();
        sys.refresh_processes();

        let mut restored_count = 0;
        let mut failed_count = 0;

        // Restore all processes to normal priority
        for pid in sys.processes().keys() {
            match set_process_priority(pid.as_u32(), PriorityClass::Normal) {
                Ok(_) => restored_count += 1,
                Err(_) => failed_count += 1,
            }
        }

        self.active = false;

        Ok(DeactivationResult {
            processes_restored: restored_count,
            failed_count,
        })
    }

    /// Check if a process name matches a game process
    fn is_game_process(&self, name: &str) -> bool {
        self.game_processes.iter()
            .any(|game| game.eq_ignore_ascii_case(name))
    }

    /// Check if any game processes are currently running
    #[allow(dead_code)]
    pub fn are_games_running(&self) -> bool {
        let mut sys = System::new_all();
        sys.refresh_processes();

        sys.processes().values().any(|process| {
            let name = process.name().to_string();
            self.is_game_process(&name)
        })
    }
}

/// Result of game mode activation
#[derive(Debug, Clone)]
pub struct ActivationResult {
    #[allow(dead_code)]
    pub games_detected: Vec<(u32, String)>,  // Detected game PIDs and names
    pub game_count: usize,
    pub background_processes_optimized: usize,
    pub failed_count: usize,
    pub memory_cleaned: Option<CleanStats>,
}

impl ActivationResult {
    pub fn summary(&self) -> String {
        let mut msg = format!(
            "Games detected: {} | Background optimized: {}",
            self.game_count,
            self.background_processes_optimized
        );

        if let Some(mem_stats) = &self.memory_cleaned {
            msg.push_str(&format!(" | Memory: {}", mem_stats.summary()));
        }

        if self.failed_count > 0 {
            msg.push_str(&format!(" | Failed: {}", self.failed_count));
        }

        msg
    }
}

/// Result of game mode deactivation
#[derive(Debug, Clone)]
pub struct DeactivationResult {
    pub processes_restored: usize,
    pub failed_count: usize,
}

impl DeactivationResult {
    pub fn summary(&self) -> String {
        format!(
            "Processes restored: {} | Failed: {}",
            self.processes_restored,
            self.failed_count
        )
    }
}

/// Auto-detect common game processes
#[allow(dead_code)]
pub fn detect_common_games() -> Vec<String> {
    vec![
        // Common game launchers
        "minecraft.exe".to_string(),
        "minecraftlauncher.exe".to_string(),
        "javaw.exe".to_string(),
        "steam.exe".to_string(),
        "epicgameslauncher.exe".to_string(),
        "battle.net.exe".to_string(),
        "upc.exe".to_string(), // Ubisoft Connect

        // Common game executables
        "valorant.exe".to_string(),
        "league of legends.exe".to_string(),
        "cs2.exe".to_string(),
        "dota2.exe".to_string(),
        "fortnitegame.exe".to_string(),
        "overwatch.exe".to_string(),
        "apex.exe".to_string(),
        "r5apex.exe".to_string(),
        "gta5.exe".to_string(),
        "skyrim.exe".to_string(),
        "skyrimse.exe".to_string(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_mode_creation() {
        let games = vec!["test.exe".to_string()];
        let gm = GameMode::new(
            games,
            PriorityClass::High,
            PriorityClass::BelowNormal,
            true,
            true,
        );

        assert!(!gm.active);
        assert_eq!(gm.game_processes.len(), 1);
    }

    #[test]
    fn test_is_game_process() {
        let games = vec!["minecraft.exe".to_string()];
        let gm = GameMode::new(
            games,
            PriorityClass::High,
            PriorityClass::BelowNormal,
            false,
            false,
        );

        assert!(gm.is_game_process("minecraft.exe"));
        assert!(gm.is_game_process("Minecraft.exe"));
        assert!(!gm.is_game_process("notepad.exe"));
    }

    #[test]
    fn test_detect_common_games() {
        let games = detect_common_games();
        assert!(!games.is_empty());
        assert!(games.iter().any(|g| g.contains("minecraft")));
    }
}
