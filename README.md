# ForgeX

> **Forge your character.**

ForgeX is a personal character-development and behavioral accountability system designed around one simple principle:

> **Character is built by repeatedly doing what you said you would do, especially when you don't feel like doing it.**

ForgeX is not a productivity tracker or habit blocker. It connects commitments, real-world behavior, consequences, rewards, recovery, and consistency into an empirical feedback loop:

$$\text{Commitment} \longrightarrow \text{Choice} \longrightarrow \text{Consequence} \longrightarrow \text{Feedback} \longrightarrow \text{Adaptation} \longrightarrow \text{Character}$$

---

## Architecture

ForgeX is organized as a modular Rust workspace:

```text
ForgeX/
├── Cargo.toml             # Root workspace definition
├── apps/
│   └── mobile/            # Flutter cross-platform mobile client (Android & Desktop)
├── crates/
│   ├── core/              # Domain models (Commitments, Activities, Penalties, Save Days, Config)
│   ├── storage/           # Embedded SQLite persistence & schema migrations via rusqlite
│   ├── scoring/           # Avoidance detection, penalty formulas, recovery engine, character metrics
│   ├── ffi/               # C/Dart shared library (libforgex_ffi.so) for native offline execution
│   ├── server/            # Async REST sync daemon with pairing authentication
│   └── cli/               # forgex binary with rich terminal UI, status dashboard, and commands
```

---

## Core Mechanics

### 1. The Penalty Model
When a commitment is missed, ForgeX calculates bounded consequences based on behavioral context:
$$\text{Raw Penalty} = \text{Importance} \times \text{Effort} \times \text{Repetition} \times \text{EntertainmentFactor}$$
$$\text{Actual Penalty} = \min(\text{Raw Penalty}, \text{MaxDailyPenalty})$$

- **Repetition**: Consecutive skip count $[1, 5]$.
- **Entertainment Factor**: Bounded $[1, 5]$ derived from logged entertainment overlapping or near ($\pm 30$m) the scheduled task window.
- **Persistent Penalties**: Consecutive days hitting the maximum daily penalty extend the duration of restricted days rather than multiplying daily burden indefinitely.

### 2. Recovery Over Perfection
Failure is expected. The goal is to recover quickly.
- Completing a task earns recovery points ($\text{Effort} \times 15$ minutes) that directly reduce remaining penalty minutes and restricted days.
- Successful task completion grants entertainment allowance bonuses ($\text{Effort} \times 5$ minutes).

### 3. Save Days
Consistency builds resilience:
- Every 7 consecutive successful days earn **+1 Save Day** (stored up to a maximum of 3).
- When a commitment is missed, an available Save Day can absorb the failure with zero penalty, protecting your streak.
- High-importance tasks ($\ge 3/5$) automatically consume an available Save Day by default, with manual override flags (`--use-save-day` or `--no-save-day`).

### 4. Character Profile
ForgeX tracks 5 core behavioral indicators:
- **Consistency**: Percentage of commitments fulfilled.
- **Reliability**: First-time completion rate without prior avoidance.
- **Resilience**: Speed of recovery after misses.
- **Self-Control**: Absence of impulsive entertainment during commitments.
- **Discipline**: Follow-through on high-effort and high-importance promises.

---

## Installation & Getting Started

### Building from Source
Ensure Rust is installed (`rustc >= 1.80`):

```bash
cargo build --release
```

The compiled binary will be located at `target/release/forgex`.

### Initializing ForgeX
Initialize the system with sample daily commitments and a starter Save Day:

```bash
forgex init --sample
```

View your dashboard:

```bash
forgex status
```

---

## Command Reference

| Command | Description |
| :--- | :--- |
| `forgex status` | Displays the daily dashboard (commitments, behavior, penalties, Save Days) |
| `forgex init [--sample]` | Initializes database and configuration |
| `forgex task add` | Creates a commitment (`-t "Title" -d 45 -i 4 -e 4`) |
| `forgex task list` | Lists scheduled and historical commitments |
| `forgex task start <ID>` | Marks a commitment as active |
| `forgex task complete <ID>` | Marks commitment complete, earning recovery reductions and screen time |
| `forgex task miss <ID>` | Records a miss, running avoidance detection and consequence formulas |
| `forgex activity log` | Logs activity (`-a "Firefox" --detail "youtube.com" -d 30 -c high`) |
| `forgex activity list` | Lists logged activities |
| `forgex character` | Displays your character profile metrics and level |
| `forgex recover` | Shows active restrictions and available tasks to recover freedom |
| `forgex evaluate [--auto-miss]`| Checks for past-due commitments exceeding the grace period |
| `forgex serve [--port 8080]` | Runs local network sync daemon for mobile and cross-device sync |
| `forgex config show / set` | Views and adjusts configuration |

---

## Cross-Platform Mobile App (Android)

The Flutter mobile client is located in [`apps/mobile`](file:///media/pirate/Shared/currently%20working/ForgeX/apps/mobile).

### Running on Android
1. Start the sync daemon on your laptop:
   ```bash
   forgex serve
   ```
2. Note the **Pairing Token** and your laptop's Wi-Fi IP address printed in the terminal.
3. Launch the Android app:
   ```bash
   cd apps/mobile
   flutter run -d android
   ```
4. Open the **Settings** tab in the app, enter your laptop's URL and pairing token, and tap **Save & Connect**. Your commitments, penalties, and activities now synchronize in real time.

---

## Global Options
- `--db <PATH>` / `FORGEX_DB`: Override SQLite database location (default: `~/.local/share/forgex/forgex.db`).
- `--config <PATH>` / `FORGEX_CONFIG`: Override configuration file location (default: `~/.config/forgex/config.toml`).
- `--json`: Output machine-readable JSON for integration into scripts or status bars.
