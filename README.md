# Mouser-RS

> A native Linux daemon and GUI for Logitech HID++ mice: button remapping, gesture control, DPI tuning, SmartShift, and more. Written in Rust.

---

## Features

- **Button Remapping** - Map any mouse button to keyboard shortcuts, media keys, browser actions, or custom key sequences.
- **Gesture Control** - Hold a button and swipe in a direction (up / down / left / right) to trigger configurable actions.
- **DPI Control** - Set and persist DPI directly via the HID++ protocol.
- **SmartShift** - Toggle and tune Logitech's SmartShift (free-spin ↔ ratchet scroll wheel) threshold.
- **Horizontal Scroll** - Map horizontal scroll tilt to browser Back / Forward or any key combo. Configurable threshold and inversion.
- **Vertical Scroll Inversion** - Optionally invert the scroll wheel direction.
- **Battery Monitor** - Real-time battery level display in the GUI for wireless mice.
- **Bluetooth & USB Receiver** - Supports devices connected via a Logitech Unifying / Bolt receiver or directly over Bluetooth.
- **Persistent Device Cache** - Paired devices are remembered across Bluetooth disconnections. Connection state updates live; devices never disappear from the GUI just because BT is off.
- **Profiles** - Multiple named profiles with per-app automatic switching and a configurable active profile.
- **System Tray** - Minimize to system tray; restore or quit from the tray menu.
- **Single Instance** - Launching a second instance brings the existing window to front instead of starting a duplicate process.
- **Auto-updater** - Built-in update checker with optional automatic install.
- **Multi-language UI** - Translation system with per-locale string files.
- **WSL2 Compatible** - Automatically falls back to software rendering when running under WSL2.

---

## Architecture

```mermaid
graph TD
    %% Styling
    classDef bin fill:#e3f2fd,stroke:#1565c0,stroke-width:2px,color:#0d47a1;
    classDef gui fill:#e8f5e9,stroke:#2e7d32,stroke-width:2px,color:#1b5e20;
    classDef engine fill:#fff3e0,stroke:#ef6c00,stroke-width:2px,color:#e65100;
    classDef os fill:#fafafa,stroke:#9e9e9e,stroke-dasharray: 5 5,color:#424242;

    %% Nodes
    subgraph App ["mouser-rs (Binary)"]
        Main["main.rs (CLI / Socket Guard / Tray)"]
    end
    class Main bin;

    subgraph GUI ["mouser_gui (egui frontend)"]
        MouserApp["MouserApp (Root View)"]
        Theme["Theme / Styles"]
        MouseUI["Mouse UI (DPI/SmartShift)"]
        SettingsUI["Settings & Profiles UI"]
        Trans["Translation / i18n"]
        
        MouserApp --> Theme
        MouserApp --> MouseUI
        MouserApp --> SettingsUI
        MouserApp --> Trans
    end
    class MouserApp,Theme,MouseUI,SettingsUI,Trans gui;

    subgraph Engine ["mouser_engine (HID++ backend)"]
        Core["Engine Core (Event Loop / State)"]
        Config["Config (JSON Persistence)"]
        HIDPP["HID++ Client (DPI / Battery)"]
        BT["BlueZ Bluetooth Helper"]
        Receiver["Unifying/Bolt USB Receiver"]
        MouseHook["Mouse Hook (evdev interception)"]
        KeyHook["Keyboard Hook (modifier tracking)"]
        Simulator["Key Simulator (uinput output)"]
        AppDetect["AppDetector (X11 / XWayland / KDE / GNOME)"]

        Core --> Config
        Core --> HIDPP
        Core --> BT
        Core --> Receiver
        Core --> MouseHook
        Core --> KeyHook
        Core --> Simulator
        Core --> AppDetect
    end
    class Core,Config,HIDPP,BT,Receiver,MouseHook,KeyHook,Simulator,AppDetect engine;

    subgraph OS ["Linux OS / Hardware Interfaces"]
        DevHID["/dev/hidraw* (Logitech Mice)"]
        DevInput["/dev/input/event* (evdev inputs)"]
        UInput["/dev/uinput (virtual inputs)"]
        BlueZ["BlueZ D-Bus Daemon"]
        WindowSys["Windowing System (X11/XWayland)"]
    end
    class DevHID,DevInput,UInput,BlueZ,WindowSys os;

    %% Connections
    Main -->|Initializes & Starts| Core
    Main -->|Launches egui| MouserApp
    MouserApp -->|Reads state & commands| Core
    
    %% Core/Engine to OS/HW Connections
    HIDPP <-->|Read/Write HID++| DevHID
    Receiver <-->|Register Devices| DevHID
    BT <-->|D-Bus API| BlueZ
    MouseHook <-->|Intercept evdev| DevInput
    KeyHook <-->|Monitor modifiers| DevInput
    Simulator -->|Inject keystrokes| UInput
    AppDetect -->|Active Window API| WindowSys
```

The project is a Cargo workspace with three crates:

```
mouse/
├── src/               # Binary entry point (main.rs)
│   └── main.rs        # CLI args, single-instance guard, engine init, GUI launch, tray
├── engine/            # mouser_engine - HID++ backend library
│   └── src/
│       ├── lib.rs          # Engine struct, background threads, event loop
│       ├── config.rs       # Config / Profile / Settings data structures + JSON persistence
│       ├── hidpp.rs        # HID++ protocol implementation (DPI, SmartShift, battery …)
│       ├── bluetooth.rs    # BlueZ D-Bus helpers, device discovery
│       ├── receiver.rs     # Unifying / Bolt USB receiver support
│       ├── mouse_hook.rs   # evdev mouse event interception
│       ├── keyboard_hook.rs# evdev keyboard monitoring (modifier state)
│       ├── key_simulator.rs# uinput virtual keyboard / mouse output
│       ├── app_detector.rs # Active window / process detection (X11)
│       ├── battery.rs      # Battery polling via HID++
│       ├── cache.rs        # Persistent paired-device cache (survives BT off)
│       ├── updater.rs      # HTTP update check & installer
│       └── worker.rs       # Thread-pool helpers
└── gui/               # mouser_gui - egui frontend library
    └── src/
        ├── lib.rs           # MouserApp root, view routing, close behaviour
        ├── theme.rs         # Design tokens (colors, typography, spacing, window size)
        ├── top_bar.rs       # Title-bar / navigation bar component
        ├── mouse_ui.rs      # Per-device customisation view (buttons, DPI, SmartShift)
        ├── settings.rs      # Application settings page (scrollable)
        ├── empty_state.rs   # No-device / unpaired state screen
        ├── select_connection.rs # Device picker / connection chooser
        └── translation.rs   # i18n string lookup
```

### Key design decisions

| Concern | Approach |
|---|---|
| Backend / GUI isolation | `mouser_engine` is a plain library; `mouser_gui` depends on it but never the reverse |
| Thread safety | `Engine` wraps `Arc<EngineInner>`; all mutable state behind `Mutex` or `AtomicBool` |
| Config hot-reload | `Engine::reload_config()` picks up a freshly-written `config.json` without restart |
| Device persistence | `cache.rs` writes paired devices to disk; GUI reads from cache, not from live BT query |
| Single instance | Unix domain socket at `~/.config/Mouser/mouser.sock`; second launch sends `SHOW` and exits |

---

## Requirements

| Dependency | Notes |
|---|---|
| Linux (X11 / XWayland) | Wayland dynamic window hiding not supported by winit; XWayland is used automatically |
| Rust ≥ 1.75 | Stable toolchain |
| `libhidapi-dev` | HID++ communication |
| `libudev-dev` | evdev / uinput |
| `libgtk-3-dev` | System tray support |
| `pkgconf` / `pkg-config` | Build-time dependency resolution |

Install build dependencies on Debian/Ubuntu:

```sh
sudo apt install libhidapi-dev libudev-dev libgtk-3-dev pkg-config build-essential
```

---

## Linux Permissions Setup

Mouser-RS reads HID raw devices and synthesises input events via `uinput`. Without the correct udev rules these nodes are only accessible as root.

Run the bundled installer **once** (after building) to install the rules without needing to run the app as root:

```sh
cd packaging/linux
sudo sh install-linux-permissions.sh
```

This copies `69-mouser-logitech.rules` to `/etc/udev/rules.d/`, reloads udev, and loads the `uinput` kernel module. Reconnect your mouse after the script completes.

---

## Building

```sh
# Development build
cargo build

# Optimised release build (LTO, strip, abort-on-panic)
cargo build --release
```

The binary is placed at `target/release/mouser-rs`.

---

## Running

```sh
# Launch GUI (default)
./target/release/mouser-rs

# Enable verbose debug logging
./target/release/mouser-rs --debug

# Headless daemon mode (no GUI, no tray)
./target/release/mouser-rs --daemon

# Show help
./target/release/mouser-rs --help
```

Launching a second instance while one is already running will bring the existing window to the foreground instead of opening a duplicate.

---

## Configuration

Settings are persisted to:

```
~/.config/Mouser/config.json
```

Logs are written to the platform log directory (typically `~/.local/share/Mouser/` or `~/.config/Mouser/`) and rotated at 5 MB, keeping the five most recent files.

### Config structure overview

```jsonc
{
  "version": 1,
  "active_profile": "Default",
  "profiles": {
    "Default": {
      "label": "Default",
      "apps": [],
      "mappings": {
        "middle": "none",
        "xbutton1": "alt_tab",
        "xbutton2": "alt_tab",
        "hscroll_left": "browser_back",
        "hscroll_right": "browser_forward",
        "mode_shift": "switch_scroll_mode"
        // … gesture directions per button …
      }
    }
  },
  "settings": {
    "dpi": 1000,
    "smart_shift_enabled": true,
    "smart_shift_threshold": 50,
    "invert_vscroll": false,
    "invert_hscroll": false,
    "hscroll_threshold": 3,
    "start_minimized": false,
    "appearance_mode": "dark",
    "language": "en",
    "accent_color": "#00BFA5"
  }
}
```

---

## Project Layout At A Glance

```
mouse/
├── Cargo.toml              # Workspace root + binary crate manifest
├── Cargo.lock
├── src/main.rs             # Binary entry point
├── engine/                 # mouser_engine library crate
├── gui/                    # mouser_gui library crate
├── assets/
│   ├── fonts/              # Bundled typefaces
│   └── images/             # UI images / icons
└── packaging/
    └── linux/
        ├── 69-mouser-logitech.rules      # udev rules for hidraw + uinput
        └── install-linux-permissions.sh  # One-shot permission installer
```

---

## License

This project is private / proprietary. All rights reserved.
