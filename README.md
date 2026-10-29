# ForgeX

> **Forge your character through empirical accountability.**

[![Build Status](https://img.shields.io/badge/build-passing-brightgreen)](#testing--development)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org/)
[![Flutter](https://img.shields.io/badge/flutter-3.x-blue.svg)](https://flutter.dev/)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](#)

ForgeX is a personal character-development and behavioral accountability system designed around one core truth:

> **Character is built by repeatedly doing what you said you would do, especially when you don't feel like doing it.**

ForgeX is neither a passive habit tracker nor a simple site blocker. Instead, it creates an empirical behavioral feedback loop that links your declared commitments, desktop and mobile activity, consequences, recovery, and character progression:

$$\text{Commitment} \longrightarrow \text{Choice} \longrightarrow \text{Consequence} \longrightarrow \text{Feedback} \longrightarrow \text{Adaptation} \longrightarrow \text{Character}$$

---

## Table of Contents

- [Architecture](#architecture)
- [Core Behavioral Mechanics](#core-behavioral-mechanics)
  - [1. The Penalty Model](#1-the-penalty-model)
  - [2. Recovery Over Perfection](#2-recovery-over-perfection)
  - [3. Save Days & Streak Protection](#3-save-days--streak-protection)
  - [4. Five-Factor Character Profile](#4-five-factor-character-profile)
- [Installation & Quick Start](#installation--quick-start)
  - [Prerequisites](#prerequisites)
  - [Building from Source](#building-from-source)
  - [Initializing the Database](#initializing-the-database)
- [Interactive Terminal User Interface (TUI)](#interactive-terminal-user-interface-tui)
  - [TUI Keybindings & Controls](#tui-keybindings--controls)
- [Linux Activity Watcher](#linux-activity-watcher)
  - [Classification Rules](#classification-rules)
  - [Running the Watcher](#running-the-watcher)
  - [Running as a Systemd User Service](#running-as-a-systemd-user-service)
- [Command-Line Interface (CLI) Reference](#command-line-interface-cli-reference)
  - [Global Flags](#global-flags)
  - [Command Breakdown](#command-breakdown)
- [REST Sync Daemon & API](#rest-sync-daemon--api)
  - [Daemon Configuration](#daemon-configuration)
  - [REST API Endpoints](#rest-api-endpoints)
- [Cross-Platform Mobile App (Flutter)](#cross-platform-mobile-app-flutter)
  - [Features](#features)
  - [Running on Android or Desktop](#running-on-android-or-desktop)
  - [Dual Architecture: REST Sync vs. Native FFI](#dual-architecture-rest-sync-vs-native-ffi)
- [Configuration Reference](#configuration-reference)
- [Testing & Development](#testing--development)

---

## Architecture

ForgeX is organized as a high-performance modular Rust workspace accompanied by a Flutter client:

```text
ForgeX/
├── Cargo.toml             # Root workspace definition
├── apps/
│   └── mobile/            # Flutter cross-platform mobile client (Android & Desktop)
├── crates/
│   ├── core/              # Domain models (Commitments, Activities, Penalties, Save Days, Config, Events)
│   ├── storage/           # Embedded SQLite persistence & migrations via rusqlite
│   ├── scoring/           # Avoidance detection, penalty formulas, recovery engine, activity classifier
│   ├── ffi/               # C/Dart shared library (libforgex_ffi.so) for native offline execution
│   ├── server/            # Async REST sync daemon with token-based pairing authentication
│   └── cli/               # forgex binary: Ratatui TUI, background window watcher, and CLI commands
```

---

## Core Behavioral Mechanics

### 1. The Penalty Model
When a commitment is missed, ForgeX calculates bounded consequences based on the gravity of the task and your behavioral context at the time:

$$\text{Raw Penalty} = \text{Importance} \times \text{Effort} \times \text{Repetition} \times \text{EntertainmentFactor}$$
$$\text{Actual Penalty} = \min(\text{Raw Penalty}, \text{MaxDailyPenalty})$$

- **Importance**: User score $[1, 5]$.
- **Effort**: Expected cognitive or physical energy $[1, 5]$.
- **Repetition**: Consecutive skip counter $[1, 5]$.
- **Entertainment Factor**: Bounded $[1, 5]$ derived from logged entertainment overlapping or near ($\pm 30\text{ min}$) the scheduled commitment window. Avoidance (indulging in entertainment during a scheduled commitment) amplifies the penalty.
- **Persistent Penalties**: Reaching the maximum daily penalty cap across consecutive days extends the duration of restricted days rather than multiplying daily burden indefinitely.

### 2. Recovery Over Perfection
Failure is expected. The primary objective is to recover rapidly.
- Completing a task earns **Recovery Points** ($\text{Effort} \times 15\text{ minutes}$) that directly pay down remaining penalty minutes and restricted days.
- Successful task completion grants **Entertainment Allowance Bonuses** ($\text{Effort} \times 5\text{ minutes}$).

### 3. Save Days & Streak Protection
Consistency builds resilience against unpredictable emergencies:
- Every 7 consecutive successful days earn **+1 Save Day** (stored up to a maximum of 3).
- When a commitment is missed, an available Save Day can absorb the failure with zero penalty, protecting your active streak.
- High-importance tasks ($\ge 3/5$) automatically consume an available Save Day by default, with manual override flags (`--use-save-day` or `--no-save-day`).

### 4. Five-Factor Character Profile
ForgeX quantifies character development using 5 empirical behavioral indicators:
- **Consistency**: Percentage of declared commitments fulfilled on schedule.
- **Reliability**: First-time completion rate without prior avoidance or rescheduling.
- **Resilience**: Speed of recovery after misses and restricted periods.
- **Self-Control**: Absence of impulsive entertainment consumption during scheduled commitment windows.
- **Discipline**: Follow-through on high-effort and high-importance commitments.

---

## Installation & Quick Start

### Prerequisites
- **Rust toolchain**: `rustc >= 1.80` and `cargo` ([rustup.rs](https://rustup.rs/))
- **SQLite**: bundled statically via `rusqlite`
- **Linux Window Query Tools** (for activity watcher): `hyprctl` (Hyprland) or `xprop` (X11 / XWayland)
- **Flutter SDK** (optional, for mobile/desktop UI): Flutter 3.x with Android SDK or Linux desktop toolchain

### Building from Source

Build all workspace crates in release mode:

```bash
cargo build --release
```

The compiled CLI binary will be located at:
```text
target/release/forgex
```

Optional: Install the binary into your Cargo bin path (`~/.cargo/bin/forgex`):
```bash
cargo install --path crates/cli
```

To compile the native C/Dart FFI shared library used by the Flutter client:
```bash
cargo build -p forgex-ffi --release
```
Outputs `target/release/libforgex_ffi.so` on Linux (`.dylib` on macOS, `.dll` on Windows).

### Initializing the Database

Initialize your ForgeX database and configuration with starter sample tasks and 1 Save Day:

```bash
forgex init --sample
```

---

## Interactive Terminal User Interface (TUI)

Running `forgex` without subcommands launches the full-screen terminal dashboard powered by [Ratatui](https://github.com/ratatui-org/ratatui):

```bash
forgex
```

### Features
- **Integrated Live Background Watcher**: Continuously monitors the focused desktop window (Hyprland / Wayland / X11), auto-classifies your activity, updates the header in real-time, and flushes activity sessions directly to SQLite.
- **Tabs & Views**:
  - `[1] Dashboard`: Today's commitments, active penalties, streak status, live activity stream, and daily summary.
  - `[2] Commitments`: Interactive commitment manager with status filters, instant start/complete/miss/delete actions.
  - `[3] Behavior`: Activity log viewer showing categorized screen time (Productive, Essential, Medium, High).
  - `[4] Recovery`: Active penalty minutes, restricted day count, and available recovery tasks.
  - `[5] Character`: Five-factor behavioral radar profile, character score, and level progression.
  - `[6] Settings`: Interactive configuration editor for penalty thresholds and allowances.

### TUI Keybindings & Controls

| Key | Context | Action |
| :--- | :--- | :--- |
| `1` – `6` | Global | Jump directly to section (Dashboard, Commitments, Behavior, Recovery, Character, Settings) |
| `Tab` / `Shift+Tab` | Global / Forms | Cycle through sections or move between form inputs |
| `n` | Global | Open modal to create a new commitment |
| `l` | Global | Open modal to log an activity manually |
| `?` | Global | Open built-in interactive User Guide modal |
| `q` / `Ctrl+C` | Global | Exit ForgeX (cleanly flushes in-progress activity sessions) |
| `↑` / `↓` / `j` / `k` | Commitments / Behavior | Navigate through list items |
| `s` | Commitments | Start the selected commitment |
| `c` | Commitments | Complete the selected commitment (earns recovery points) |
| `m` | Commitments | Mark commitment missed (runs consequence calculations) |
| `d` | Commitments | Delete the selected commitment |
| `f` | Commitments | Cycle status filter (`all` $\rightarrow$ `planned` $\rightarrow$ `active` $\rightarrow$ `completed` $\rightarrow$ `missed`) |
| `Enter` | Settings | Edit selected configuration value |
| `Esc` | Modals / Forms | Cancel or dismiss active modal overlay |

---

## Linux Activity Watcher

ForgeX includes an automatic window watcher capable of tracking your active desktop window on Linux (Hyprland native via `hyprctl activewindow -j`, or X11 / XWayland via `xprop`).

### Classification Rules

The scoring engine classifies active windows into 4 categories:

| Category | Typical Windows & Applications | Consequence Impact |
| :--- | :--- | :--- |
| **Productive** | VS Code, Neovim, Zed, Cursor, Alacritty, Kitty, Ghostty, Terminal, Obsidian, Logseq, docs.rs, GitHub, StackOverflow, Rust docs | Protects focus; no avoidance penalty |
| **Essential** | Bitwarden, KeePassXC, 1Password, system settings, authentication prompts, banking portals | Neutral; necessary tasks |
| **Medium Entertainment** | YouTube videos, Netflix, Twitch, Reddit, Spotify, VLC, MPV | Triggers avoidance factor if during commitments |
| **High Entertainment** | YouTube Shorts, TikTok, Instagram Reels, Twitter/X, Steam, Heroic, Lutris, games | Maximum avoidance factor ($5\times$ penalty multiplier) |

> [!NOTE]
> **Continuous Session Tracking**: The watcher intelligently tracks continuous entertainment sessions across video title and URL changes (e.g. browsing between YouTube videos or Shorts) without prematurely splitting sessions into fragmented 2-second chunks.

### Running the Watcher

Run directly in a terminal:
```bash
# Poll every 2 seconds, flush completed sessions every 30 seconds
forgex watch --interval 2 --flush 30
```

Silent mode (suppresses stdout logging):
```bash
forgex watch --quiet
```

### Running as a Systemd User Service

To run the watcher continuously in the background on user login:

1. Create `~/.config/systemd/user/forgex-watcher.service`:

```ini
[Unit]
Description=ForgeX Linux Desktop Activity Watcher
After=graphical-session.target

[Service]
Type=simple
ExecStart=%h/.cargo/bin/forgex watch --quiet
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

2. Reload and enable the service:

```bash
systemctl --user daemon-reload
systemctl --user enable --now forgex-watcher
```

3. Check service status or logs:

```bash
systemctl --user status forgex-watcher
journalctl --user -u forgex-watcher -f
```

---

## Command-Line Interface (CLI) Reference

### Global Flags

- `--db <PATH>` / `FORGEX_DB`: Override SQLite database location (default: `~/.local/share/forgex/forgex.db`).
- `--config <PATH>` / `FORGEX_CONFIG`: Override configuration file location (default: `~/.config/forgex/config.toml`).
- `--json`: Output command responses in structured JSON format for scripting or custom status bars (Polybar, Waybar, etc.).

### Command Breakdown

#### System & Dashboard
```bash
# Launch interactive TUI
forgex

# Print terminal CLI dashboard
forgex status
forgex status --json

# Initialize database and configuration
forgex init
forgex init --sample
```

#### Commitment Management (`forgex task`)
```bash
# Add a new commitment
forgex task add -t "Deep Work: Rust Engine" -d 60 -i 5 -e 4 -c "Coding" -r "daily"

# List commitments (optionally filter: planned, active, completed, missed)
forgex task list
forgex task list -s planned

# Start commitment by ID or ID prefix
forgex task start a1b2

# Complete commitment (awards recovery points and entertainment allowance)
forgex task complete a1b2

# Mark commitment as missed (calculates consequences)
forgex task miss a1b2

# Force or prevent Save Day consumption on miss
forgex task miss a1b2 --use-save-day
forgex task miss a1b2 --no-save-day

# Delete a commitment
forgex task delete a1b2
```

#### Activity Logging (`forgex activity`)
```bash
# Manually log an activity record
forgex activity log -a "Firefox" --detail "youtube.com" -d 45 -c high

# List logged activities
forgex activity list
```

#### Evaluation & Character
```bash
# Evaluate overdue commitments past their grace period
forgex evaluate
forgex evaluate --auto-miss

# View character profile radar metrics and tier
forgex character

# View active restrictions and recovery paths
forgex recover
```

#### Configuration (`forgex config`)
```bash
# View all current settings
forgex config show

# Update a setting
forgex config set max_daily_penalty_mins 180
forgex config set grace_period_mins 20
```

---

## REST Sync Daemon & API

ForgeX includes an embedded asynchronous HTTP/JSON daemon powered by [Axum](https://github.com/tokio-rs/axum) and [Tokio](https://tokio.rs/) for synchronizing with mobile and other local devices.

### Daemon Configuration

Launch the daemon on your primary machine:

```bash
# Bind to all local interfaces on port 8080 (auto-generates a secure pairing token)
forgex serve

# Custom port and token
forgex serve --bind 0.0.0.0 --port 9000 --token "my-custom-token"
```

The daemon outputs your local IP addresses and the **Pairing Token** upon launch:
```text
✔ ForgeX Sync Daemon online at http://0.0.0.0:8080
  Local IP:      http://192.168.1.100:8080
  Pairing Token: f8a1c9...
```

### REST API Endpoints

All data endpoints require authentication via Bearer token (`Authorization: Bearer <TOKEN>`) or query parameter (`?token=<TOKEN>`).

| Method | Endpoint | Description |
| :--- | :--- | :--- |
| `GET` | `/api/health` | Health check (public, returns online status) |
| `GET` | `/api/status` | Daily summary, commitments, activities, penalties, Save Days |
| `GET` | `/api/commitments` | List all commitments |
| `POST` | `/api/commitments` | Create a new commitment |
| `POST` | `/api/commitments/:id/start` | Start a commitment |
| `POST` | `/api/commitments/:id/complete` | Complete a commitment |
| `POST` | `/api/commitments/:id/miss` | Mark commitment as missed |
| `POST` | `/api/activities` | Log activity record |
| `GET` | `/api/character` | Retrieve 5-factor character profile & scores |
| `GET` | `/api/recover` | Retrieve active penalties and recovery recommendations |
| `POST` | `/api/sync` | Bi-directional batch synchronization payload |

---

## Cross-Platform Mobile App (Flutter)

The mobile client is located in [`apps/mobile`](apps/mobile). It provides a full mobile-optimized interface with dedicated tabs:
- **Dashboard**: Daily status, active penalties, today's commitments, and entertainment gauge.
- **Commitments**: Task list with interactive quick actions (Start, Complete, Miss).
- **Activity**: Screen time ledger categorized by productivity and entertainment.
- **Character**: Five-factor radar breakdown, behavioral level, and consistency rates.
- **Settings**: Connection settings, host IP, pairing token status, and sync trigger.

### Running on Android or Desktop

1. Start the sync daemon on your development host:
   ```bash
   forgex serve
   ```
2. Navigate to the mobile app directory and fetch dependencies:
   ```bash
   cd apps/mobile
   flutter pub get
   ```
3. Run on connected Android device or emulator:
   ```bash
   flutter run -d android
   ```
   Or run directly as a native Linux desktop application:
   ```bash
   flutter run -d linux
   ```
4. Open the **Settings** tab in the app, enter your host URL (`http://<HOST_IP>:8080`) and **Pairing Token**, and tap **Save & Connect**.

> [!TIP]
> The Android manifest (`AndroidManifest.xml`) is pre-configured with `android:usesCleartextTraffic="true"` to enable seamless pairing over local Wi-Fi networks without requiring custom TLS certificates.

### Dual Architecture: REST Sync vs. Native FFI

The Flutter app supports two operational modes:
1. **Network Sync Mode**: Connects via `ForgeXApiService` to the local REST sync daemon (`forgex serve`).
2. **Native Offline FFI Mode**: Directly links to `libforgex_ffi.so` via `dart:ffi` for embedded offline execution without network dependency.

---

## Configuration Reference

The configuration file is stored at `~/.config/forgex/config.toml` (or overridden via `--config` / `FORGEX_CONFIG`).

```toml
# Maximum penalty minutes that can be assigned in a single day
max_daily_penalty_mins = 120

# Maximum number of Save Days that can be banked
save_days_max = 3

# Consecutive successful days required to earn +1 Save Day
save_day_streak_required = 7

# Importance score (1-5) at or above which Save Days auto-absorb failures
auto_save_day_importance_threshold = 3

# Base daily entertainment allowance in minutes
default_entertainment_allowance_mins = 60

# Grace period in minutes after scheduled start before a task is overdue
grace_period_mins = 15
```

---

## Testing & Development

Run all Rust unit, integration, scoring, and end-to-end tests:

```bash
cargo test --workspace
```

Run Flutter widget and unit tests:

```bash
cd apps/mobile
flutter test
```

Format code and check linting:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

---

## License

ForgeX is open-source software licensed under either the [MIT License](LICENSE-MIT) or the [Apache License 2.0](LICENSE-APACHE) at your option.
