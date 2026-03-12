use eframe::egui;
use egui::{Color32, RichText};
use std::time::Instant;
use crate::config::{Config, PriorityClass, Theme};
use crate::memory::monitor::MemoryStatus;
use crate::process::manager::{ProcessInfo, get_all_processes, set_process_priority, kill_process};
use crate::process::manager::PriorityClass as WinPriorityClass;
use crate::process::gamemode::GameMode;

// Constants for monitoring intervals
const MONITORING_INTERVAL_SECS: u64 = 1;
const PROCESS_REFRESH_INTERVAL_SECS: u64 = 5;

// Professional color scheme for EndlessOpt - Glassmorphism style
struct Colors {
    primary: Color32,
    secondary: Color32,
    cta: Color32,
    success: Color32,
    warning: Color32,
    error: Color32,
    background: Color32,
    surface: Color32,
    glass: Color32,
    text: Color32,
    text_secondary: Color32,
    border: Color32,
    shadow: Color32,
    accent: Color32,
}

impl Colors {
    fn modern_dark() -> Self {
        Colors {
            primary: Color32::from_rgb(15, 23, 42),       // Navy dark (#0F172A)
            secondary: Color32::from_rgb(51, 65, 85),     // Navy medium (#334155)
            cta: Color32::from_rgb(3, 105, 161),         // System blue (#0369A1)
            success: Color32::from_rgb(34, 197, 94),      // Success green (#22C55E)
            warning: Color32::from_rgb(251, 191, 36),     // Warning amber (#FBBF24)
            error: Color32::from_rgb(239, 68, 68),        // Error red (#EF4444)
            background: Color32::from_rgb(2, 6, 23),      // Very dark (#020617)
            surface: Color32::from_rgb(30, 41, 59),      // Surface (#1E293B)
            glass: Color32::from_rgba_unmultiplied(255, 255, 255, 20), // Glass white with low opacity
            text: Color32::from_rgb(248, 250, 252),      // Off-white (#F8FAFC)
            text_secondary: Color32::from_rgb(148, 163, 184), // Slate gray (#94A3B8)
            border: Color32::from_rgba_unmultiplied(255, 255, 255, 25), // Subtle border
            shadow: Color32::from_rgba_unmultiplied(0, 0, 0, 51),    // Shadow
            accent: Color32::from_rgb(56, 189, 248),     // Sky blue (#38BDF8)
        }
    }

    fn modern_light() -> Self {
        Colors {
            primary: Color32::from_rgb(15, 23, 42),       // Navy dark (#0F172A)
            secondary: Color32::from_rgb(51, 65, 85),     // Navy medium (#334155)
            cta: Color32::from_rgb(3, 105, 161),         // System blue (#0369A1)
            success: Color32::from_rgb(34, 197, 94),      // Success green (#22C55E)
            warning: Color32::from_rgb(251, 191, 36),     // Warning amber (#FBBF24)
            error: Color32::from_rgb(239, 68, 68),        // Error red (#EF4444)
            background: Color32::from_rgb(248, 250, 252), // Light gray (#F8FAFC)
            surface: Color32::from_rgb(255, 255, 255),    // White (#FFFFFF)
            glass: Color32::from_rgba_unmultiplied(255, 255, 255, 230), // Glass white
            text: Color32::from_rgb(15, 23, 42),          // Dark navy (#0F172A)
            text_secondary: Color32::from_rgb(71, 85, 105), // Slate (#475569)
            border: Color32::from_rgba_unmultiplied(0, 0, 0, 25),      // Subtle border
            shadow: Color32::from_rgba_unmultiplied(0, 0, 0, 25),     // Shadow
            accent: Color32::from_rgb(56, 189, 248),      // Sky blue (#38BDF8)
        }
    }
}

/// Main application tabs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Optimize,
    Processes,
    Settings,
}

/// Main application state
pub struct EndlessOptApp {
    // Tab management
    current_tab: Tab,

    // System monitoring
    cpu_usage: f32,
    memory_usage: f32,
    memory_status: Option<MemoryStatus>,
    last_update: Instant,
    sys: sysinfo::System,  // Reusable system instance

    // Process list
    processes: Vec<ProcessInfo>,
    selected_process: Option<usize>,
    process_filter: String,
    show_blacklisted: bool,

    // Settings
    config: Config,
    config_modified: bool,

    // UI state
    status_message: String,
    status_color: Color32,
    optimization_in_progress: bool,
    game_mode_active: bool,

    // Optimization results
    last_optimization_result: Option<String>,

    // Color scheme
    colors: Colors,
    is_dark_mode: bool,

    // Version info
    version: String,

    // Confirmation dialogs
    show_kill_confirmation: bool,
    process_to_kill: Option<(u32, String)>,  // (pid, name)
}

impl EndlessOptApp {
    /// Create a new application instance
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load configuration
        let config = Config::load().unwrap_or_default();

        // Determine dark mode
        let is_dark_mode = match config.theme {
            Theme::Dark => true,
            Theme::Light => false,
            Theme::System => {
                // Try to detect system preference
                cc.egui_ctx.style().visuals.dark_mode
            }
        };

        // Set initial theme and colors
        setup_theme(&cc.egui_ctx, &config.theme);
        let colors = if is_dark_mode {
            Colors::modern_dark()
        } else {
            Colors::modern_light()
        };

        Self {
            current_tab: Tab::Dashboard,
            cpu_usage: 0.0,
            memory_usage: 0.0,
            memory_status: None,
            last_update: Instant::now(),
            sys: sysinfo::System::new_all(),
            processes: Vec::new(),
            selected_process: None,
            process_filter: String::new(),
            show_blacklisted: false,
            config,
            config_modified: false,
            status_message: "Ready".to_string(),
            status_color: Color32::GRAY,
            optimization_in_progress: false,
            game_mode_active: false,
            last_optimization_result: None,
            colors,
            is_dark_mode,
            version: env!("CARGO_PKG_VERSION").to_string(),
            show_kill_confirmation: false,
            process_to_kill: None,
        }
    }

    /// Update system monitoring data
    fn update_monitoring(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update).as_secs() >= MONITORING_INTERVAL_SECS {
            // Update memory status
            if let Ok(status) = MemoryStatus::get() {
                self.memory_usage = status.memory_load as f32;
                self.memory_status = Some(status);
            }

            // Update CPU usage using sysinfo
            self.sys.refresh_cpu();
            self.cpu_usage = self.sys.cpus().iter()
                .map(|c| c.cpu_usage())
                .sum::<f32>() / self.sys.cpus().len() as f32;

            // Update process list occasionally (only when viewing Processes tab)
            if self.current_tab == Tab::Processes &&
               now.duration_since(self.last_update).as_secs() >= PROCESS_REFRESH_INTERVAL_SECS {
                if let Ok(processes) = get_all_processes(&self.config.blacklisted_processes) {
                    self.processes = processes;
                }
            }

            self.last_update = now;
        }
    }

    /// Show status message
    fn show_status(&mut self, message: &str, color: Color32) {
        self.status_message = message.to_string();
        self.status_color = color;
    }

    /// Perform full optimization
    fn perform_full_optimization(&mut self) {
        self.optimization_in_progress = true;
        self.show_status("Performing full optimization...", Color32::YELLOW);

        let mut results = Vec::new();

        // PCL-style advanced memory optimization
        if let Ok(result) = crate::memory::optimizer::optimize_memory_pcl_style(
            crate::memory::optimizer::OptimizationLevel::Aggressive
        ) {
            results.push(format!("Memory: {}", result.summary()));
        }

        // Optimize processes
        if let Ok(stats) = crate::process::manager::optimize_processes(
            &self.config.game_processes,
            &self.config.blacklisted_processes,
            self.config.game_priority.clone().into(),
            self.config.bg_priority.clone().into(),
        ) {
            results.push(format!("Processes: {}", stats.summary()));
        }

        // Clean temp files
        if let Ok(stats) = crate::utils::cleaner::clean_temp_files() {
            results.push(format!("Temp Files: {}", stats.summary()));
        }

        // Release network resources
        if self.config.net_optimize {
            if let Ok(stats) = crate::utils::cleaner::release_network_resources() {
                results.push(format!("Network: {}", stats.summary()));
            }
        }

        self.last_optimization_result = Some(results.join("\n"));
        self.optimization_in_progress = false;
        self.show_status("Full optimization complete!", Color32::GREEN);
    }

    /// Activate game mode
    #[allow(dead_code)]
    fn activate_game_mode(&mut self) {
        let mut game_mode = GameMode::new(
            self.config.game_processes.clone(),
            self.config.game_priority.clone().into(),
            self.config.bg_priority.clone().into(),
            self.config.mem_clean,
            self.config.net_optimize,
        );

        match game_mode.activate() {
            Ok(result) => {
                self.game_mode_active = true;
                self.show_status(
                    &format!("Game mode activated! {}", result.summary()),
                    Color32::GREEN
                );
            }
            Err(e) => {
                self.show_status(
                    &format!("Failed to activate game mode: {}", e),
                    Color32::RED
                );
            }
        }
    }

    /// Deactivate game mode
    #[allow(dead_code)]
    fn deactivate_game_mode(&mut self) {
        let mut game_mode = GameMode::new(
            self.config.game_processes.clone(),
            self.config.game_priority.clone().into(),
            self.config.bg_priority.clone().into(),
            false,
            false,
        );

        match game_mode.deactivate() {
            Ok(result) => {
                self.game_mode_active = false;
                self.show_status(
                    &format!("Game mode deactivated! {}", result.summary()),
                    Color32::GREEN
                );
            }
            Err(e) => {
                self.show_status(
                    &format!("Failed to deactivate game mode: {}", e),
                    Color32::RED
                );
            }
        }
    }
}

impl eframe::App for EndlessOptApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update monitoring data
        self.update_monitoring();

        // Set window properties
        ctx.input(|i| {
            let _ = i.viewport().close_requested();
        });

        // Top panel with tabs
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new("EndlessOpt").size(24.0));
                ui.separator();

                ui.selectable_value(&mut self.current_tab, Tab::Dashboard, "Dashboard");
                ui.selectable_value(&mut self.current_tab, Tab::Optimize, "Optimize");
                ui.selectable_value(&mut self.current_tab, Tab::Processes, "Processes");
                ui.selectable_value(&mut self.current_tab, Tab::Settings, "Settings");

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.game_mode_active {
                        ui.colored_label(Color32::GREEN, RichText::new("Game Mode Active").size(14.0));
                    }
                });
            });
        });

        // Status bar
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("CPU: {:.1}%", self.cpu_usage)).size(12.0));
                ui.separator();
                ui.label(RichText::new(format!("Memory: {:.1}%", self.memory_usage)).size(12.0));
                ui.separator();
                ui.colored_label(self.status_color, RichText::new(&self.status_message).size(12.0));
            });
        });

        // Main content
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                Tab::Dashboard => self.show_dashboard(ui, ctx),
                Tab::Optimize => self.show_optimize(ui),
                Tab::Processes => self.show_processes(ui),
                Tab::Settings => self.show_settings(ui),
            }
        });

        // Kill confirmation dialog
        if self.show_kill_confirmation {
            self.show_kill_confirmation_dialog(ctx);
        }
    }
}

// Implement tab rendering methods
impl EndlessOptApp {
    fn show_dashboard(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        // Glassmorphism background
        ui.painter().rect_filled(
            ui.available_rect_before_wrap(),
            0.0,
            self.colors.background
        );

        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Modern title with glass effect
            egui::Frame::none()
                .fill(self.colors.glass)
                .stroke(egui::Stroke::new(1.0, self.colors.border))
                .rounding(16.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);
                        ui.heading(RichText::new("EndlessOpt").size(36.0)
                            .color(self.colors.text)
                            .strong());
                        ui.label(RichText::new("System Optimizer").size(16.0)
                            .color(self.colors.text_secondary));
                        ui.label(RichText::new(format!("v{}", self.version)).size(12.0)
                            .color(self.colors.text_secondary));
                        ui.add_space(15.0);
                    });
                });

            ui.add_space(30.0);

            // Status metrics row - Professional dashboard layout
            ui.horizontal(|ui| {
                // CPU Metric Card
                show_metric_card(
                    ui,
                    &self.colors,
                    "CPU Usage",
                    &format!("{:.1}%", self.cpu_usage),
                    "Processor",
                    get_usage_color(self.cpu_usage, &self.colors),
                    None,
                );

                ui.add_space(20.0);

                // Memory Metric Card
                let memory_subtitle = if let Some(ref status) = self.memory_status {
                    format!("{}\nRAM", MemoryStatus::format_bytes(status.total_phys - status.avail_phys))
                } else {
                    "RAM".to_string()
                };

                show_metric_card(
                    ui,
                    &self.colors,
                    "Memory Usage",
                    &format!("{:.1}%", self.memory_usage),
                    &memory_subtitle,
                    get_usage_color(self.memory_usage, &self.colors),
                    None,
                );

                ui.add_space(20.0);

                // Game Mode Status Card
                let game_mode_value = if self.game_mode_active { "Active" } else { "Inactive" };
                let game_mode_color = if self.game_mode_active {
                    self.colors.success
                } else {
                    self.colors.text_secondary
                };

                show_metric_card(
                    ui,
                    &self.colors,
                    "Game Mode",
                    game_mode_value,
                    "Performance",
                    game_mode_color,
                    if self.game_mode_active { Some(self.colors.success) } else { None },
                );
            });

            ui.add_space(30.0);

            // Primary CTA Button - Professional styling
            let button_response = ui.add_sized(
                [240.0, 50.0],
                egui::Button::new(
                    RichText::new("Full Optimize").size(18.0)
                        .color(self.colors.text)
                        .strong()
                )
                .fill(self.colors.cta)
                .rounding(8.0)
            );

            // Hover effect simulation (visual feedback)
            if button_response.hovered() {
                ui.ctx().output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
            }

            if button_response.clicked() {
                self.perform_full_optimization();
            }

            ui.add_space(20.0);

            // Status Panel - Glassmorphism
            if !self.status_message.is_empty() || self.last_optimization_result.is_some() {
                egui::Frame::none()
                    .fill(self.colors.glass)
                    .stroke(egui::Stroke::new(1.0, self.colors.border))
                    .rounding(12.0)
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 2.0),
                        blur: 6.0,
                        spread: 0.0,
                        color: self.colors.shadow,
                    })
                    .show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.add_space(15.0);

                            if !self.status_message.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Status:").size(14.0)
                                        .color(self.colors.text_secondary)
                                        .strong());
                                    ui.label(RichText::new(&self.status_message).size(14.0)
                                        .color(self.status_color));
                                });
                                ui.add_space(8.0);
                            }

                            if let Some(ref result) = self.last_optimization_result {
                                ui.label(RichText::new("Last Optimization:").size(14.0)
                                    .color(self.colors.text_secondary)
                                    .strong());
                                ui.label(RichText::new(result).size(13.0)
                                    .color(self.colors.text));
                            }

                            ui.add_space(15.0);
                        });
                    });
            }
        });
    }

    fn show_optimize(&mut self, ui: &mut egui::Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(20.0);

            // Section Title - Professional styling
            egui::Frame::none()
                .fill(self.colors.glass)
                .stroke(egui::Stroke::new(1.0, self.colors.border))
                .rounding(12.0)
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(15.0);
                        ui.heading(RichText::new("System Optimization").size(28.0)
                            .color(self.colors.text)
                            .strong());
                        ui.label(RichText::new("Quick actions to optimize your system").size(14.0)
                            .color(self.colors.text_secondary));
                        ui.add_space(15.0);
                    });
                });

            ui.add_space(25.0);

            if self.optimization_in_progress {
                // Loading state with professional styling
                egui::Frame::none()
                    .fill(self.colors.glass)
                    .stroke(egui::Stroke::new(1.0, self.colors.border))
                    .rounding(12.0)
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(RichText::new("Optimizing...").size(16.0)
                                .color(self.colors.text_secondary));
                            ui.add_space(20.0);
                        });
                    });
                return;
            }

            // Action Buttons Grid - Professional glassmorphism
            ui.horizontal(|ui| {
                // Clean Memory Button
                let clean_memory_response = ui.add_sized(
                    [170.0, 100.0],
                    egui::Button::new(
                        RichText::new("Clean\nMemory").size(16.0)
                            .color(self.colors.text)
                            .strong()
                    )
                    .fill(self.colors.secondary)
                    .rounding(12.0)
                );

                if clean_memory_response.hovered() {
                    ui.ctx().output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
                }

                if clean_memory_response.clicked() {
                    match crate::memory::optimizer::optimize_memory_pcl_style(
                        crate::memory::optimizer::OptimizationLevel::Aggressive
                    ) {
                        Ok(result) => {
                            self.show_status(&result.user_friendly_summary(), self.colors.success);
                        }
                        Err(e) => {
                            self.show_status(&format!("Failed to clean memory: {}", e), self.colors.error);
                        }
                    }
                }

                ui.add_space(20.0);

                // Optimize Processes Button
                let optimize_processes_response = ui.add_sized(
                    [170.0, 100.0],
                    egui::Button::new(
                        RichText::new("Optimize\nProcesses").size(16.0)
                            .color(self.colors.text)
                            .strong()
                    )
                    .fill(self.colors.primary)
                    .rounding(12.0)
                );

                if optimize_processes_response.hovered() {
                    ui.ctx().output_mut(|output| output.cursor_icon = egui::CursorIcon::PointingHand);
                }

                if optimize_processes_response.clicked() {
                    match crate::process::manager::optimize_processes(
                        &self.config.game_processes,
                        &self.config.blacklisted_processes,
                        self.config.game_priority.clone().into(),
                        self.config.bg_priority.clone().into(),
                    ) {
                        Ok(stats) => {
                            self.show_status(&format!("Processes optimized: {}", stats.summary()), self.colors.success);
                        }
                        Err(e) => {
                            self.show_status(&format!("Failed to optimize processes: {}", e), self.colors.error);
                        }
                    }
                }
            });

            ui.add_space(25.0);

            // Status Panel - Professional feedback
            if !self.status_message.is_empty() {
                egui::Frame::none()
                    .fill(self.colors.glass)
                    .stroke(egui::Stroke::new(1.0, self.status_color))
                    .rounding(8.0)
                    .shadow(egui::epaint::Shadow {
                        offset: egui::vec2(0.0, 2.0),
                        blur: 6.0,
                        spread: 0.0,
                        color: self.colors.shadow,
                    })
                    .show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(12.0);
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Status:").size(14.0)
                                    .color(self.colors.text_secondary)
                                    .strong());
                                ui.label(RichText::new(&self.status_message).size(14.0)
                                    .color(self.status_color)
                                    .strong());
                            });
                            ui.add_space(12.0);
                        });
                    });
            }
        });
    }

    fn show_processes(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Process Manager").size(28.0));
            ui.add_space(10.0);

            // Filter and controls
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.text_edit_singleline(&mut self.process_filter);
                ui.checkbox(&mut self.show_blacklisted, "Show Blacklisted");

                if ui.button("Refresh").clicked() {
                    if let Ok(processes) = get_all_processes(&self.config.blacklisted_processes) {
                        self.processes = processes;
                        self.show_status("Process list refreshed", Color32::GREEN);
                    }
                }
            });

            ui.add_space(10.0);

            // Process table
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Header
                ui.horizontal(|ui| {
                    ui.label(RichText::new("PID").strong());
                    ui.label(RichText::new("Name").strong());
                    ui.label(RichText::new("CPU %").strong());
                    ui.label(RichText::new("Memory").strong());
                    ui.label(RichText::new("Priority").strong());
                });
                ui.separator();

                // Process rows
                for (idx, process) in self.processes.iter().enumerate() {
                    // Apply filter
                    if !self.process_filter.is_empty()
                        && !process.name.to_lowercase().contains(&self.process_filter.to_lowercase())
                    {
                        continue;
                    }

                    // Skip blacklisted if not showing
                    if process.is_blacklisted && !self.show_blacklisted {
                        continue;
                    }

                    ui.horizontal(|ui| {
                        ui.label(format!("{}", process.pid));
                        ui.label(&process.name);
                        ui.label(format!("{:.1}%", process.cpu_usage));
                        ui.label(format!("{:.1} MB", process.memory_usage as f32 / 1024.0));
                        ui.label(process.priority.as_str());

                        // Context menu
                        if ui.button("...").clicked() {
                            self.selected_process = Some(idx);
                        }
                    });
                }
            });

            // Process context menu
            if let Some(idx) = self.selected_process {
                if idx < self.processes.len() {
                    // Clone the process data to avoid borrow issues
                    let process_clone = self.processes[idx].clone();

                    egui::Window::new(format!("Process: {}", process_clone.name))
                        .collapsible(false)
                        .resizable(false)
                        .show(ui.ctx(), |ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("PID: {}", process_clone.pid));
                                ui.label(format!("CPU: {:.1}%", process_clone.cpu_usage));
                                ui.label(format!("Memory: {:.1} MB", process_clone.memory_usage as f32 / 1024.0));
                                ui.label(format!("Priority: {}", process_clone.priority.as_str()));

                                ui.separator();

                                ui.label("Set Priority:");
                                let pid = process_clone.pid;
                                if ui.button("Idle").clicked() {
                                    let _ = set_process_priority(pid, WinPriorityClass::Idle);
                                }
                                ui.horizontal(|ui| {
                                    if ui.button("Below Normal").clicked() {
                                        let _ = set_process_priority(pid, WinPriorityClass::BelowNormal);
                                    }
                                    if ui.button("Normal").clicked() {
                                        let _ = set_process_priority(pid, WinPriorityClass::Normal);
                                    }
                                });
                                ui.horizontal(|ui| {
                                    if ui.button("Above Normal").clicked() {
                                        let _ = set_process_priority(pid, WinPriorityClass::AboveNormal);
                                    }
                                    if ui.button("High").clicked() {
                                        let _ = set_process_priority(pid, WinPriorityClass::High);
                                    }
                                });

                                ui.separator();

                                if ui.button(RichText::new("Kill Process").color(Color32::RED)).clicked() {
                                    // Show confirmation dialog
                                    self.process_to_kill = Some((process_clone.pid, process_clone.name.clone()));
                                    self.show_kill_confirmation = true;
                                }

                                if ui.button("Close").clicked() {
                                    self.selected_process = None;
                                }
                            });
                        });
                }
            }
        });
    }

    fn show_settings(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading(RichText::new("Settings").size(28.0));
            ui.add_space(20.0);

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Auto-optimization
                ui.group(|ui| {
                    ui.heading(RichText::new("Auto-Optimization").size(16.0));
                    ui.checkbox(&mut self.config.auto_optimize, "Enable Auto-Optimization");

                    if self.config.auto_optimize {
                        ui.add(egui::Slider::new(&mut self.config.auto_interval, 1..=60)
                            .text("Interval (minutes)"));
                    }

                    self.config_modified = true;
                });

                ui.add_space(10.0);

                // Game Mode Settings
                ui.group(|ui| {
                    ui.heading(RichText::new("Game Mode Settings").size(16.0));
                    ui.checkbox(&mut self.config.auto_game_mode, "Auto-activate Game Mode");

                    ui.horizontal(|ui| {
                        ui.label("Game Priority:");
                        egui::ComboBox::from_id_source("game_priority")
                            .selected_text(format!("{:?}", self.config.game_priority))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::Idle, "Idle");
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::BelowNormal, "Below Normal");
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::Normal, "Normal");
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::AboveNormal, "Above Normal");
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::High, "High");
                                ui.selectable_value(&mut self.config.game_priority, PriorityClass::Realtime, "Realtime");
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Background Priority:");
                        egui::ComboBox::from_id_source("bg_priority")
                            .selected_text(format!("{:?}", self.config.bg_priority))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::Idle, "Idle");
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::BelowNormal, "Below Normal");
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::Normal, "Normal");
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::AboveNormal, "Above Normal");
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::High, "High");
                                ui.selectable_value(&mut self.config.bg_priority, PriorityClass::Realtime, "Realtime");
                            });
                    });

                    ui.checkbox(&mut self.config.mem_clean, "Clean Memory in Game Mode");
                    ui.checkbox(&mut self.config.net_optimize, "Optimize Network in Game Mode");

                    self.config_modified = true;
                });

                ui.add_space(10.0);

                // Game Processes
                ui.group(|ui| {
                    ui.heading(RichText::new("Game Processes").size(16.0));
                    ui.label("Enter game process names (comma-separated, e.g., minecraft.exe, steam.exe):");

                    let game_processes_str = self.config.game_processes.join(", ");
                    let mut game_processes_text = game_processes_str;

                    if ui.text_edit_multiline(&mut game_processes_text).changed() {
                        self.config.game_processes = game_processes_text
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        self.config_modified = true;
                    }

                    ui.label(format!("{} game processes configured", self.config.game_processes.len()));
                });

                ui.add_space(10.0);

                // Process Blacklist
                ui.group(|ui| {
                    ui.heading(RichText::new("Process Blacklist").size(16.0));
                    ui.label("Enter process names to exclude from optimization (comma-separated):");

                    let blacklist_str = self.config.blacklisted_processes.join(", ");
                    let mut blacklist_text = blacklist_str;

                    if ui.text_edit_multiline(&mut blacklist_text).changed() {
                        self.config.blacklisted_processes = blacklist_text
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect();
                        self.config_modified = true;
                    }

                    ui.label(format!("{} processes blacklisted", self.config.blacklisted_processes.len()));
                });

                ui.add_space(10.0);

                // Theme
                ui.group(|ui| {
                    ui.heading(RichText::new("Appearance").size(16.0));

                    ui.horizontal(|ui| {
                        ui.label("Theme:");
                        if ui.selectable_value(&mut self.config.theme, Theme::Light, "Light").changed() {
                            setup_theme(ui.ctx(), &Theme::Light);
                            self.config_modified = true;
                        }
                        if ui.selectable_value(&mut self.config.theme, Theme::Dark, "Dark").changed() {
                            setup_theme(ui.ctx(), &Theme::Dark);
                            self.config_modified = true;
                        }
                        if ui.selectable_value(&mut self.config.theme, Theme::System, "System").changed() {
                            setup_theme(ui.ctx(), &Theme::System);
                            self.config_modified = true;
                        }
                    });
                });

                ui.add_space(20.0);

                // Save button
                ui.horizontal(|ui| {
                    if ui.button("Save Configuration").clicked() {
                        match self.config.save() {
                            Ok(_) => {
                                self.show_status("Configuration saved", Color32::GREEN);
                                self.config_modified = false;
                            }
                            Err(e) => {
                                self.show_status(&format!("Failed to save: {}", e), Color32::RED);
                            }
                        }
                    }

                    if ui.button("Reset to Defaults").clicked() {
                        self.config = Config::default();
                        setup_theme(ui.ctx(), &self.config.theme);
                        self.config_modified = true;
                        self.show_status("Configuration reset to defaults", Color32::YELLOW);
                    }
                });
            });
        });
    }

    fn show_kill_confirmation_dialog(&mut self, ctx: &egui::Context) {
        let (pid, process_name) = match &self.process_to_kill {
            Some((pid, name)) => (*pid, name.clone()),
            None => return,
        };

        egui::Window::new("Confirm Process Termination")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(10.0);
                    ui.label(RichText::new("Are you sure you want to kill this process?")
                        .size(16.0)
                        .color(self.colors.text)
                        .strong());
                    ui.add_space(10.0);
                    ui.label(RichText::new(format!("Process: {}", process_name))
                        .size(14.0)
                        .color(self.colors.error));
                    ui.add_space(5.0);
                    ui.label(RichText::new("This action cannot be undone.")
                        .size(12.0)
                        .color(self.colors.text_secondary));
                    ui.add_space(15.0);

                    ui.horizontal(|ui| {
                        if ui.button("Cancel").clicked() {
                            self.show_kill_confirmation = false;
                            self.process_to_kill = None;
                        }

                        ui.add_space(10.0);

                        if ui.button(RichText::new("Kill Process").color(Color32::RED)).clicked() {
                            match kill_process(pid) {
                                Ok(_) => {
                                    self.show_status(
                                        &format!("Process {} killed", process_name),
                                        Color32::GREEN
                                    );
                                    self.selected_process = None;
                                    // Refresh process list
                                    if let Ok(processes) = get_all_processes(&self.config.blacklisted_processes) {
                                        self.processes = processes;
                                    }
                                }
                                Err(e) => {
                                    self.show_status(
                                        &format!("Failed to kill process: {}", e),
                                        Color32::RED
                                    );
                                }
                            }
                            self.show_kill_confirmation = false;
                            self.process_to_kill = None;
                        }
                    });

                    ui.add_space(10.0);
                });
            });
    }
}

// Helper functions

/// Show a unified metric card with glassmorphism styling
fn show_metric_card(
    ui: &mut egui::Ui,
    colors: &Colors,
    title: &str,
    value: &str,
    subtitle: &str,
    value_color: Color32,
    border_color: Option<Color32>,
) {
    egui::Frame::none()
        .fill(colors.glass)
        .stroke(egui::Stroke::new(1.0, border_color.unwrap_or(colors.border)))
        .rounding(12.0)
        .shadow(egui::epaint::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 8.0,
            spread: 0.0,
            color: colors.shadow,
        })
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(15.0);
                ui.label(RichText::new(title).size(14.0)
                    .color(colors.text_secondary));
                ui.add_space(8.0);
                ui.label(RichText::new(value).size(32.0)
                    .color(value_color)
                    .strong());
                ui.label(RichText::new(subtitle).size(12.0)
                    .color(colors.text_secondary));
                ui.add_space(15.0);
            });
        });
}

fn get_usage_color(usage: f32, colors: &Colors) -> Color32 {
    if usage < 50.0 {
        colors.success
    } else if usage < 80.0 {
        colors.warning
    } else {
        colors.error
    }
}

fn setup_theme(ctx: &egui::Context, theme: &Theme) {
    match theme {
        Theme::Light => {
            ctx.set_visuals(egui::Visuals::light());
        }
        Theme::Dark => {
            ctx.set_visuals(egui::Visuals::dark());
        }
        Theme::System => {
            // Check system preference
            let is_dark = ctx.style().visuals.dark_mode;
            if is_dark {
                ctx.set_visuals(egui::Visuals::dark());
            } else {
                ctx.set_visuals(egui::Visuals::light());
            }
        }
    }
}

// Conversion implementations

impl From<PriorityClass> for crate::process::manager::PriorityClass {
    fn from(p: PriorityClass) -> Self {
        match p {
            PriorityClass::Idle => crate::process::manager::PriorityClass::Idle,
            PriorityClass::BelowNormal => crate::process::manager::PriorityClass::BelowNormal,
            PriorityClass::Normal => crate::process::manager::PriorityClass::Normal,
            PriorityClass::AboveNormal => crate::process::manager::PriorityClass::AboveNormal,
            PriorityClass::High => crate::process::manager::PriorityClass::High,
            PriorityClass::Realtime => crate::process::manager::PriorityClass::Realtime,
        }
    }
}
