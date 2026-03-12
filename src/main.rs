#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // Hide console in release builds

mod common;
mod config;
mod gui;
mod memory;
mod process;
mod utils;
mod validation;

use eframe::egui;

fn main() -> eframe::Result<()> {
    // Load icon
    let icon_data = load_icon();

    // Initialize options
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([600.0, 400.0])
            .with_icon(icon_data),
        ..Default::default()
    };

    // Run the application
    eframe::run_native(
        "EndlessOpt - System Optimizer",
        options,
        Box::new(|cc| {
            setup_fonts(&cc.egui_ctx);
            Box::new(gui::EndlessOptApp::new(cc))
        }),
    )
}

fn load_icon() -> egui::IconData {
    // Try to load the new professional icon file
    let icon_path = "assets/icon.ico";

    if let Ok(icon_data) = std::fs::read(icon_path) {
        // The new icon is 512x512 with multiple sizes embedded
        egui::IconData {
            rgba: icon_data,
            width: 512u32,
            height: 512u32,
        }
    } else {
        // Fallback to a simple colored icon if file not found
        create_fallback_icon()
    }
}

fn create_fallback_icon() -> egui::IconData {
    // Create a simple but professional fallback icon
    let width = 256u32;
    let height = 256u32;
    let mut rgba = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;

            // Create a purple-pink gradient (matching our app colors)
            let norm_x = x as f32 / width as f32;
            let norm_y = y as f32 / height as f32;

            // Gradient from purple (#667eea) to pink (#F093FB)
            let r = (102.0 + norm_x * 138.0) as u8;
            let g = (126.0 + norm_x * 21.0) as u8;
            let b = (234.0 - norm_x * 87.0) as u8;

            // Add depth with Y variation (clamp to u8 range)
            let depth = (norm_y * 30.0) as u8;
            let depth_r = r.saturating_sub(depth);
            let depth_g = g.saturating_sub(depth);
            let depth_b = b.saturating_sub(depth);

            rgba[idx] = depth_r;
            rgba[idx + 1] = depth_g;
            rgba[idx + 2] = depth_b;
            rgba[idx + 3] = 255;
        }
    }

    egui::IconData { rgba, width, height }
}

fn setup_fonts(ctx: &egui::Context) {
    // Use the default egui fonts - they have excellent Unicode support
    // and work reliably across all platforms
    let fonts = egui::FontDefinitions::default();
    ctx.set_fonts(fonts);
}
