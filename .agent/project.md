# PROJECT.md — mouser-rs (Logi Options+ clone for Linux)

Rust workspace: a Logitech HID++ daemon (`mouser_engine`) wrapped by an egui/GTK
frontend (`mouser_gui`), launched by a tiny binary crate (root `mouser-rs`).
Architecture mirrors the active project note: backend reused as-is, stdio/gRPC
bridge to the GUI, no Electron/React.

## Crates
- **root `mouser-rs`** (src/) — binary only. CLI dispatch, tray icon, signal
  handling, single-instance lock, eframe bootstrap. Depends on `engine` + `gui`.
- **`engine/`** (mouser_engine) — the daemon: HID++ device I/O, input
  simulation (hooks + virtual devices), app/window detection, flow/task
  switching, gRPC server. No GUI.
- **`gui/`** (mouser_gui) — egui views + widgets, theme, i18n, updater,
  desktop-app lookups. Talks to engine via `EngineClient` (gRPC) or in-process
  `Engine`.

## Entry points (read these first)
- `src/main.rs` — arg parse (`--daemon`, `--debug`, subcommands status/info/
  profile/dpi/reload/cli). Spawns tray + GUI or runs as daemon.
- `engine/src/lib.rs` — module map; `pub use engine::Engine`.
- `gui/src/lib.rs` — module map; `pub use app::MouserApp`, `views::ActiveView`.
- `gui/src/app/mod.rs` — `MouserApp` (egui lifecycle, update loop).
- `engine/src/engine/mod.rs` + `inner.rs` — `Engine` core state + event loop.

## Root binary (`src/`)
- `main.rs` — launch/CLI router, daemon vs GUI mode.
- `cli.rs` — subcommand implementations (status/info/profile/dpi/reload).
- `gui.rs` — builds/opens the egui window.
- `tray.rs` — system tray icon + menu.
- `signal.rs` — SIGINT/SIGTERM/hotplug signal handling.
- `single_instance.rs` — ensure one running instance (lock file / socket).

## engine — by module
**HID++ (Logitech protocol)**
- `hidpp/mod.rs` — public HID++ API surface.
- `hidpp/protocol.rs` — raw protocol constants/parsing (reports, feature index).
- `hidpp/device.rs` — per-device HID++ handle (open, feature queries).
- `hidpp/diversion.rs` — "diversion" rules (remap/report interception).
- `hidpp/tests.rs` — protocol unit tests.
- `receiver.rs` — unifying receiver discovery/enumerate.
- `battery.rs` — battery level polling.
- `bluetooth.rs` — bt device discovery/state.
- `cache.rs` — on-disk cache (device config / learned state).
- `config.rs` — config file load/save (5 pub fns, 5 pub types).
- `lock_ext.rs` — lock helper extension trait.

**Input simulation**
- `input/mod.rs` — module root.
- `input/mouse_hook.rs` — grabs/uinput mouse events (2 pub types).
- `input/keyboard_hook.rs` — grabs/uinput keyboard events.
- `input/simulator/mod.rs` — virtual device simulator root.
- `input/simulator/emit.rs` — low-level uinput emit.
- `input/simulator/actions.rs` — action → input mapping.
- `input/simulator/key_map.rs`, `mouse_map.rs` — key/mouse code tables.
- `input/simulator/layout_translator.rs` — layout/keymap translation.
- `input/simulator/compose.rs` — compose-key / multi-key sequences.
- `input/simulator/hangul.rs` — Hangul IME composition (2 pub types).

**App/window detection (frontmost app)**
- `detection/mod.rs` — backend selector.
- `detection/thread.rs` — polling thread.
- `detection/gnome.rs` — GNOME Shell (DBus) backend.
- `detection/kde.rs`, `sway.rs`, `hyprland.rs`, `i3.rs`, `x11.rs` — per-DE backends.
- `detection/xdotool.rs` — xdotool fallback.
- `detection/fallbacks.rs` — fallback chain.
- `detection/wlr_foreign_toplevel.rs` — wlroots toplevel-mgmt protocol.

**Flow (task/context switching)**
- `flow/mod.rs` — module root.
- `flow/topology.rs` — window/workspace topology (1 pub type).
- `flow/network.rs` — cross-machine flow over network (1 pub type).
- `flow/switching.rs` — switch logic.
- `flow/handoff.rs` — hand off context between devices (5 fns, 1 pub type).
- `flow/edge.rs` — edge-trigger detection (3 fns, 1 pub type).
- `flow/clipboard.rs` — clipboard sync (4 fns).
- `flow/channel_switch.rs` — channel switch (1 fn).

**Engine internals**
- `engine/inner.rs` — `EngineInner` state (3 pub types).
- `engine/action.rs`, `gesture.rs`, `profile.rs` — action/gesture/profile models.
- `engine/modifier_state.rs` — tracked modifier key state.
- `engine/hscroll.rs` — horizontal scroll handling.
- `engine/hotplug.rs` — device hotplug events.
- `engine/app_change.rs` — frontmost-app-change reactions.

**Bridge / IPC**
- `grpc/mod.rs` — module root.
- `grpc/server.rs` — tonic gRPC server wrapping `Engine` (generated stubs in OUT_DIR; 4 pub fns/types).
- `client.rs` — `EngineClient`: sync gRPC client mirroring `Engine` API (GUI swaps Engine<->EngineClient).
- `worker.rs` — background worker (1 fn, 2 pub types).

**Build**
- `build.rs` — build script (prost/tonic codegen for gRPC).

**Tests**
- `tests/integration.rs`, `tests/grpc_test.rs` — integration + gRPC round-trip.

## gui — by module
**App shell**
- `app/mod.rs` — `MouserApp` (2 pub types).
- `app/update.rs` — per-frame update wiring.
- `app/toast.rs`, `app/texture.rs` — toasts, texture loading.

**Views**
- `views/mod.rs` — `ActiveView` enum (1 pub type).
- `views/top_bar.rs` — top toolbar.
- `views/select_connection.rs` — connection picker.
- `views/empty_state/mod.rs` + `device_card.rs` — no-device screen (6 fns).
- `views/settings/mod.rs` — settings screen + sections: theme, profiles,
  language, updates.
- `views/customization/mod.rs` + `sidebar.rs` — customization layout.
- `views/customization/tabs/` — buttons, keyboard, point_scroll, flow, profiles tabs.
- `views/customization/mappings/` — button_options, button_keys, gesture_presets.
- `views/customization/popups/` — action_list, add_app_modal, gesture_config,
  record_shortcut, thumbwheel.
- `views/actions_ring.rs` — cursor-centred radial "Actions Ring" overlay.

**Widgets**
- `widgets/mod.rs` — root.
- `widgets/battery.rs`, `status_pill.rs`, `confirm_dialog.rs` — basic widgets.
- `widgets/connection_icon.rs` + `widgets/icons/*` — connection + sidebar +
  settings + misc icon painters (sidebar_icons.rs: 7 fns).

**Support**
- `theme.rs` — color theme tokens (24 fns).
- `translation.rs` — i18n strings (1 pub type).
- `icon_loader.rs` — load app/device icons (5 fns).
- `desktop_apps.rs` — desktop-app lookup (3 fns, 1 pub type).
- `updater.rs` — self-update (2 pub types).

## How to reduce run time (build/test)
- Prefer `cargo check --workspace` over `cargo build` when only type-checking.
- Build a single crate: `cargo build -p mouser_engine` / `-p mouser_gui`.
- Edit-and-check loop: `cargo check -p <crate>` is far faster than full build.
- Reuse `target/`: never delete it; incremental + `codegen-units=1`/`lto` are
  release-only (debug builds skip them, stay fast).
- Install `sccache` (`RUSTC_WRAPPER=sccache`) to cache across clean rebuilds.
- `engine/build.rs` runs protoc/tonic codegen — if unchanged .proto, it still
  reruns; touch-only rebuilds are cheap, but avoid `cargo clean`.
- Run one test file: `cargo test -p mouser_engine --test integration`.
- Daemon changes only touch `engine/`; GUI-only changes touch `gui/` — keep
  edits in one crate to minimize what recompiles.

## Conventions for agents
- GUI must not reimplement engine logic — call `Engine`/`EngineClient`.
- New detection backend → register in `detection/mod.rs` selector.
- Public API changes in `Engine` likely need a mirror in `client.rs`.
- The workspace has 3 crates; `git status` shows 18 modified + 1 untracked on
  branch `new_interagation`.
