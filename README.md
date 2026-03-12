<div align="center">

  ![EndlessOpt Logo](assets/icon.svg)

  # EndlessOpt

  ### ⚡ Professional Windows System Optimizer

  [![Version](https://img.shields.io/badge/version-1.0.2-blue.svg)](https://github.com/Ian-bug/endlessopt/releases/latest)
  [![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
  [![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)](https://github.com/Ian-bug/endlessopt/actions)
  [![Rust](https://img.shields.io/badge/rust-1.74+-orange.svg)](https://www.rust-lang.org/)
  [![Platform](https://img.shields.io/badge/platform-Windows%2010%2B-blue.svg)](https://www.microsoft.com/windows)

  **A comprehensive system optimization tool built with Rust, featuring a beautiful glassmorphism UI.**

  [Download](https://github.com/Ian-bug/endlessopt/releases/latest) · [Features](#-features) · [Documentation](#-documentation) · [Contributing](#-contributing)

</div>

---

## ✨ Features

### 🚀 System Optimization

- **Memory Cleaning** - Free up RAM using Windows EmptyWorkingSet API
- **Process Priority Management** - Automatically optimize process priorities
- **Temporary File Cleaning** - Remove unnecessary temp files to free disk space
- **Network Optimization** - Release DNS cache and network resources

### 🎮 Game Mode

- **Automatic Game Detection** - Detects and prioritizes game processes
- **Background Optimization** - Reduces CPU/usage interference from background apps
- **Optional Memory Cleaning** - Clean RAM during gaming sessions
- **Customizable Game List** - Add your favorite games

### 📊 Real-time Monitoring

- **Live CPU Usage** - Real-time processor monitoring with graphs
- **Memory Tracking** - Detailed memory statistics and usage trends
- **Process Management** - View, filter, and manage running processes
- **Color-coded Status** - Visual feedback for system health

### 🎨 Professional UI

- **Glassmorphism Design** - Modern frosted glass aesthetic
- **Dark/Light Themes** - Choose your preferred visual style
- **Responsive Layout** - Clean interface that adapts to your needs
- **Smooth Animations** - Professional transitions and feedback

### ⚙️ Configuration

- **Auto-optimization Scheduling** - Set intervals for automatic optimization
- **Custom Priority Classes** - Configure priorities for games and background apps
- **Process Blacklist** - Protect critical system processes from optimization
- **Persistent Settings** - Your configuration is saved automatically

---

## 🎯 Quick Start

### Option 1: Download Pre-built Binary (Recommended)

1. **Download** the latest `EndlessOpt.exe` from [Releases](https://github.com/Ian-bug/endlessopt/releases/latest)
2. **Run** the executable - no installation required!
3. **Click** "⚡ Full Optimize" on the Dashboard for comprehensive optimization

### Option 2: Build from Source

```bash
# Clone the repository
git clone https://github.com/Ian-bug/endlessopt.git
cd endlessopt

# Build the project
cargo build --release

# Run the application
./target/release/endlessopt.exe
```

---

## 📸 Screenshots

### Dashboard
![Dashboard](docs/screenshots/dashboard.png)
*Real-time system monitoring with beautiful glassmorphism design*

### Optimization
![Optimization](docs/screenshots/optimize.png)
*Quick access to all optimization features*

### Process Manager
![Process Manager](docs/screenshots/processes.png)
*View and manage running processes*

---

## 📖 Usage Guide

### System Optimization

1. **Full Optimization** (Recommended)
   - Click "⚡ Full Optimize" on the Dashboard
   - Automatically cleans memory, optimizes processes, and cleans temp files

2. **Individual Actions**
   - Go to the **Optimize** tab
   - Choose specific actions:
     - Clean Memory
     - Optimize Processes
     - Clean Temp Files
     - Release Network Resources

### Game Mode Setup

1. Navigate to **Settings** → **Game Mode Settings**
2. Add your game processes (e.g., `minecraft.exe`, `steam.exe`, `valorant.exe`)
3. Configure priorities:
   - **Game Priority**: High (recommended)
   - **Background Priority**: Below Normal (recommended)
4. Enable optional features:
   - ☑ Clean Memory in Game Mode
   - ☑ Optimize Network in Game Mode
5. Click "Save Configuration"

### Process Management

1. Go to the **Processes** tab
2. Use the filter to find specific processes
3. Click "..." on any process to:
   - Set Priority (Idle → Realtime)
   - Kill Process (with confirmation)
   - Add to Blacklist

**Note**: 26 critical system processes are protected and cannot be killed

---

## 🔧 Configuration

EndlessOpt stores configuration in `~/.endlessopt/config.json`.

### Default Configuration

```json
{
  "auto_optimize": false,
  "auto_interval": 30,
  "auto_game_mode": false,
  "game_priority": "High",
  "bg_priority": "BelowNormal",
  "mem_clean": true,
  "net_optimize": true,
  "game_processes": [
    "minecraft.exe",
    "steam.exe",
    "javaw.exe"
  ],
  "blacklisted_processes": [
    "system",
    "svchost.exe",
    "explorer.exe"
  ],
  "theme": "Dark"
}
```

### Protected Processes

The following processes are protected from termination:

**Windows System Processes:**
- System, Registry, smss.exe, csrss.exe, wininit.exe
- services.exe, lsass.exe, winlogon.exe, svchost.exe
- lsm.exe, explorer.exe, dwm.exe, audiodg.exe, spoolsv.exe

**Security Processes:**
- msmpeng.exe (Windows Defender)
- securityhealthservice.exe

**Self-Protection:**
- endlessopt.exe

---

## 💻 System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **OS** | Windows 10 (x64) | Windows 11 (x64) |
| **RAM** | 2 GB | 4 GB or more |
| **Disk Space** | 50 MB | 100 MB |
| **Permissions** | User | Administrator* |

*Administrator rights recommended for full functionality (process priority changes, system cleaning)

---

## 🏗️ Architecture

EndlessOpt is built with performance and safety in mind:

### Tech Stack

- **Rust** - Core language for memory safety and performance
- **egui/eframe** - Fast, friendly GUI framework
- **windows-rs** - Windows API bindings for system operations
- **sysinfo** - Cross-platform system information
- **serde** - Configuration serialization

### Project Structure

```
endlessopt/
├── src/
│   ├── main.rs              # Application entry point
│   ├── config/              # Configuration management
│   ├── gui/                 # User interface (egui)
│   │   ├── app.rs          # Main application state
│   │   └── tabs.rs         # Tab implementations
│   ├── memory/             # Memory monitoring & optimization
│   │   ├── monitor.rs      # Memory status tracking
│   │   └── optimizer.rs    # Memory cleaning (EmptyWorkingSet)
│   ├── process/            # Process management
│   │   ├── manager.rs      # Process enumeration & priorities
│   │   └── gamemode.rs     # Game mode optimization
│   └── utils/              # Utilities
│       └── cleaner.rs      # Temp file & network cleaning
├── assets/                  # Icons and resources
├── docs/                    # Documentation
└── tests/                   # Test suites
```

### Key Modules

| Module | Description | Lines of Code |
|--------|-------------|---------------|
| `gui/app.rs` | Main UI with glassmorphism design | 1,090 |
| `process/manager.rs` | Process management & priorities | 302 |
| `utils/cleaner.rs` | Temp file & network cleaning | 274 |
| `process/gamemode.rs` | Game mode optimization | 253 |
| `memory/optimizer.rs` | Memory cleaning implementation | 125 |
| `memory/monitor.rs` | Memory status monitoring | 107 |
| `config/mod.rs` | Configuration management | 116 |
| **Total** | **Core functionality** | **2,373** |

---

## 🧪 Testing

All tests passing (13/13):

```bash
cargo test
```

**Test Coverage:**
- ✅ Memory monitoring and optimization
- ✅ Process management and priorities
- ✅ Game mode detection and activation
- ✅ Protected process validation
- ✅ Configuration serialization
- ✅ Temporary file cleaning

---

## 📋 Changelog

### [1.0.2] - 2025-03-12

**Code Polish**
- ⚡ Fixed process index staleness bug
- 📦 Added System instance reuse for better performance
- 🎨 Created unified metric card function (reduced 100+ lines of duplicate code)
- 🔧 Reduced compiler warnings from 17 to 7
- ✅ All 13 tests passing

### [1.0.1] - 2025-03-12

**Security & Performance Update**
- 🛡️ Added protected process list (26 processes)
- 🔒 Implemented kill confirmation dialog
- ⚡ Optimized process list refresh (only when viewing Processes tab)
- 🧪 Added protected process detection test
- 📝 Fixed unused Result warnings

### [1.0.0] - 2025-03-12

**Initial Release**
- ✨ Complete system optimization features
- 🎨 Professional glassmorphism UI
- 🎮 Game mode with automatic game detection
- 📊 Real-time monitoring dashboard
- ⚙️ Comprehensive configuration system

---

## 🤝 Contributing

We welcome contributions! Please follow these guidelines:

### How to Contribute

1. **Fork** the repository
2. **Create** a feature branch (`git checkout -b feature/amazing-feature`)
3. **Make** your changes
4. **Test** thoroughly (`cargo test` and `cargo clippy`)
5. **Commit** your changes (`git commit -m 'Add amazing feature'`)
6. **Push** to the branch (`git push origin feature/amazing-feature`)
7. **Open** a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Add tests for new features
- Update documentation as needed
- Keep PRs focused and well-described
- Ensure all tests pass before submitting

### Code Style

- Use `cargo fmt` for formatting
- Run `cargo clippy` and fix warnings
- Add doc comments for public APIs
- Keep functions focused and readable

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

**Inspiration & References:**
- **[PCL-CE](https://github.com/PCL-Community/PCL-CE)** - Memory monitoring patterns and Windows API usage
- **[Process Optimizer](https://github.com/)** - Optimization logic and game mode inspiration
- **[egui](https://github.com/emilk/egui)** - Excellent GUI framework for Rust
- **[windows-rs](https://github.com/microsoft/windows-rs)** - Microsoft-maintained Windows API bindings

**Design:**
- Glassmorphism UI following [UI/UX Pro Max](https://skills.sh/) guidelines
- Color palette inspired by modern design systems

---

## ⚠️ Disclaimer

This software is provided **as-is** for educational and optimization purposes.

- **Always create system backups** before making significant changes
- **Review processes carefully** before modifying or terminating
- **Test in a safe environment** before using on production systems
- The authors are **not responsible** for any system instability or data loss

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/Ian-bug/endlessopt/issues)
- **Discussions**: [GitHub Discussions](https://github.com/Ian-bug/endlessopt/discussions)
- **Releases**: [GitHub Releases](https://github.com/Ian-bug/endlessopt/releases)

---

<div align="center">

  **Made with ❤️ in Rust**

  [⬆ Back to Top](#endlessopt)

</div>
